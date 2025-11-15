use poise::serenity_prelude as serenity;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use tokio_stream::{Stream, wrappers::ReceiverStream};
use tonic::{Request, Response, Status, transport::Server};
use tracing::{info, warn};

// Include the generated protobuf code
pub mod minecraft_bridge {
    tonic::include_proto!("minecraft_bridge");
}

use minecraft_bridge::{
    AccessStatus, ClientRegistration, EventSubscription, MinecraftServer, PlayerAccessRequest,
    PlayerAccessResponse, RegistrationResponse, ServerEvent,
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
    /// Registered clients (client_id -> list of servers)
    pub registered_clients: Arc<RwLock<std::collections::HashMap<String, Vec<MinecraftServer>>>>,
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
    async fn register_client(
        &self,
        request: Request<ClientRegistration>,
    ) -> Result<Response<RegistrationResponse>, Status> {
        let registration = request.into_inner();
        let client_id = registration.client_id.clone();
        let servers = registration.servers;

        info!(
            "[gRPC::RegisterClient] Client {} registering with {} servers",
            client_id,
            servers.len()
        );

        // Store the registered client and its servers
        let mut clients = self.state.registered_clients.write().await;
        clients.insert(client_id.clone(), servers.clone());

        info!(
            "[gRPC::RegisterClient] Client {} registered successfully",
            client_id
        );

        Ok(Response::new(RegistrationResponse { success: true }))
    }

    /// Check if a player is allowed to join a specific server
    async fn check_player_access(
        &self,
        request: Request<PlayerAccessRequest>,
    ) -> Result<Response<PlayerAccessResponse>, Status> {
        let access_request = request.into_inner();
        let client_id = access_request.client_id;
        let server_name = access_request.server_name;
        let player_ipv4 = access_request.player_ipv4;

        info!(
            "[gRPC::CheckPlayerAccess] Checking access for player {} to server {} (client: {})",
            player_ipv4, server_name, client_id
        );

        // Verify client is registered
        let clients = self.state.registered_clients.read().await;
        if !clients.contains_key(&client_id) {
            warn!(
                "[gRPC::CheckPlayerAccess] Client {} not registered",
                client_id
            );
            return Err(Status::not_found("Client not registered"));
        }

        // TODO: Implement actual access control logic
        // This is where you'd check:
        // 1. If the player's IP is in the Tailscale network
        // 2. If the player has signed up via Discord
        // 3. If the player has the required permissions/roles
        //
        // For now, we'll return a simple response
        // You can use self.state.data.db for database queries
        // You can use self.state.data.ts_client for Tailscale API calls

        // Example: Check if player IP exists in database
        let player_exists: bool = true;

        let response = if player_exists {
            PlayerAccessResponse {
                status: AccessStatus::Allowed as i32,
                reason: String::new(),
            }
        } else {
            PlayerAccessResponse {
                status: AccessStatus::RequiresSignup as i32,
                reason: "PLAYER_NOT_REGISTERED".to_string(),
            }
        };

        info!(
            "[gRPC::CheckPlayerAccess] Player {} access: {:?}",
            player_ipv4,
            AccessStatus::try_from(response.status).unwrap_or(AccessStatus::Disallowed)
        );

        Ok(Response::new(response))
    }

    /// Subscribe to server events (server-streaming)
    type SubscribeEventsStream =
        Pin<Box<dyn Stream<Item = Result<ServerEvent, Status>> + Send + 'static>>;

    async fn subscribe_events(
        &self,
        request: Request<EventSubscription>,
    ) -> Result<Response<Self::SubscribeEventsStream>, Status> {
        let subscription = request.into_inner();
        let client_id = subscription.client_id;
        let event_types = subscription.event_types;

        info!(
            "[gRPC::SubscribeEvents] Client {} subscribing to events: {:?}",
            client_id, event_types
        );

        // Verify client is registered
        let clients = self.state.registered_clients.read().await;
        if !clients.contains_key(&client_id) {
            warn!(
                "[gRPC::SubscribeEvents] Client {} not registered",
                client_id
            );
            return Err(Status::not_found("Client not registered"));
        }
        drop(clients);

        // Subscribe to the broadcast channel
        let mut rx = self.state.event_tx.subscribe();

        // Create a channel to convert broadcast to mpsc for streaming
        let (tx, stream_rx) = tokio::sync::mpsc::channel(100);

        // Spawn a task to forward events from broadcast to mpsc
        tokio::spawn(async move {
            while let Ok(event) = rx.recv().await {
                // If event_types is empty, send all events
                // Otherwise, only send events that match the requested types
                if event_types.is_empty() || event_types.contains(&event.event_type) {
                    if tx.send(Ok(event)).await.is_err() {
                        break; // Client disconnected
                    }
                }
            }
        });

        let stream = ReceiverStream::new(stream_rx);

        info!(
            "[gRPC::SubscribeEvents] Client {} subscribed successfully",
            client_id
        );

        Ok(Response::new(Box::pin(stream)))
    }
}

/// Start the gRPC server
pub async fn start_grpc_server(
    ctx: Arc<serenity::Context>,
    data: Arc<crate::Data>,
    addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create broadcast channel for events
    let (event_tx, _) = broadcast::channel::<ServerEvent>(100);

    let state = GrpcServiceState {
        ctx,
        data,
        registered_clients: Arc::new(RwLock::new(std::collections::HashMap::new())),
        event_tx: event_tx.clone(),
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
#[allow(dead_code)]
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
