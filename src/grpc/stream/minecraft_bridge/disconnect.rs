// on role update ["create", "main"]
// on disconnect ["*"]

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::grpc;
use crate::grpc::minecraft_bridge::{
    EventType, PlayerUpdateEvent, ServerEvent, server_event::EventData,
};

pub async fn guild_member_removal(
    data: Arc<crate::Data>,
    player_name: String,
    player_ipv4: String,
) {
    let _ = grpc::broadcast_event(
        &data.grpc_event_tx,
        ServerEvent {
            event_type: EventType::PlayerUpdate as i32,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64,
            target_proxy_id: None,
            event_data: Some(EventData::PlayerUpdate(PlayerUpdateEvent {
                player_name,
                player_ipv4,
            })),
        },
    )
    .await;
}
