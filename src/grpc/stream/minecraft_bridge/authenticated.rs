use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::grpc;
use crate::grpc::minecraft_bridge::{
    EventType, PlayerUpdateEvent, ServerEvent, server_event::EventData,
};

pub async fn broadcast_event(data: Arc<crate::Data>, minecraft_user_id: i64) {
    let record = sqlx::query!(
        "SELECT player_name, player_ipv4 FROM minecraft_users WHERE id = $1",
        minecraft_user_id
    )
    .fetch_one(&data.db)
    .await
    .unwrap();

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
                player_name: record.player_name,
                player_ipv4: record.player_ipv4,
            })),
        },
    )
    .await;
}
