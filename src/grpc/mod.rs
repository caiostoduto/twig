use poise::serenity_prelude as serenity;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio_stream::{Stream, wrappers::ReceiverStream};
use tonic::{Request, Response, Status, transport::Server};
use tracing::{info, warn};

// Include the generated protobuf code
pub mod minecraft_bridge {
    tonic::include_proto!("minecraft_bridge");
}

// Message handler modules
pub mod message;
pub mod stream;

use minecraft_bridge::{
    EventSubscription, PlayerAccessRequest, PlayerAccessResponse, ProxyRegistration,
    RegistrationResponse, ServerEvent,
    minecraft_bridge_server::{MinecraftBridge, MinecraftBridgeServer},
};

/// Shared state for the gRPC service
#[derive(Clone)]
pub struct GrpcServiceState {
    /// Poise framework context - contains bot client and framework data
    #[allow(dead_code)]
    pub ctx: Arc<serenity::Context>,
    /// User data from the bot
    pub data: Arc<crate::Data>,
    /// Event broadcast channel for pub/sub
    pub event_tx: broadcast::Sender<ServerEvent>,
}

/// Implementation of the MinecraftBridge gRPC service
pub struct MinecraftBridgeService {
    state: GrpcServiceState,
}

impl MinecraftBridgeService {
    pub fn new(state: GrpcServiceState) -> Self {
        Self { state }
    }
}

#[tonic::async_trait]
impl MinecraftBridge for MinecraftBridgeService {
    /// Register a Minecraft client and store its server list
    async fn register_proxy(
        &self,
        request: Request<ProxyRegistration>,
    ) -> Result<Response<RegistrationResponse>, Status> {
        message::register_proxy::register_proxy(&self.state, request).await
    }

    /// Check if a player is allowed to join a specific server
    async fn check_player_access(
        &self,
        request: Request<PlayerAccessRequest>,
    ) -> Result<Response<PlayerAccessResponse>, Status> {
        message::check_player_access::check_player_access(&self.state, request).await
    }

    /// Subscribe to server events (server-streaming)
    type SubscribeEventsStream =
        Pin<Box<dyn Stream<Item = Result<ServerEvent, Status>> + Send + 'static>>;

    async fn subscribe_events(
        &self,
        request: Request<EventSubscription>,
    ) -> Result<Response<Self::SubscribeEventsStream>, Status> {
        let subscription = request.into_inner();
        let proxy_id = subscription.proxy_id;
        let event_types = subscription.event_types;

        info!(
            "[gRPC::SubscribeEvents] Client {} subscribing to events: {:?}",
            proxy_id, event_types
        );

        // Subscribe to the broadcast channel
        let mut rx = self.state.event_tx.subscribe();

        // Create a channel to convert broadcast to mpsc for streaming
        let (tx, stream_rx) = tokio::sync::mpsc::channel(100);

        // Spawn a task to forward events from broadcast to mpsc
        tokio::spawn(async move {
            while let Ok(event) = rx.recv().await {
                // Filter by event type
                let type_matches =
                    event_types.is_empty() || event_types.contains(&event.event_type);

                // Filter by target proxy ID (None or empty = broadcast to all)
                let proxy_matches = event
                    .target_proxy_id
                    .as_ref()
                    .map(|target| target.is_empty() || target == &proxy_id)
                    .unwrap_or(true);

                if type_matches && proxy_matches {
                    if tx.send(Ok(event)).await.is_err() {
                        break; // Client disconnected
                    }
                }
            }
        });

        let stream = ReceiverStream::new(stream_rx);
        Ok(Response::new(Box::pin(stream)))
    }
}

/// Start the gRPC server
pub async fn start_grpc_server(
    ctx: Arc<serenity::Context>,
    data: Arc<crate::Data>,
    addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    let state = GrpcServiceState {
        ctx,
        data: data.clone(),
        event_tx: (*data.grpc_event_tx).clone(),
    };

    let service = MinecraftBridgeService::new(state.clone());

    info!("[gRPC] Starting gRPC server on {}", addr);

    Server::builder()
        .add_service(MinecraftBridgeServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}

/// Helper function to broadcast an event to all subscribers
pub async fn broadcast_event(
    event_tx: &broadcast::Sender<ServerEvent>,
    event: ServerEvent,
) -> Result<usize, Box<dyn std::error::Error>> {
    match event_tx.send(event) {
        Ok(receivers) => {
            info!("[gRPC] Event broadcasted to {} receivers", receivers);
            Ok(receivers)
        }

        Err(e) => {
            warn!("[gRPC] Failed to broadcast event: {}", e);
            Err(Box::new(e))
        }
    }
}
