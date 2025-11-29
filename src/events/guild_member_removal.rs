use std::sync::Arc;

use crate::{Data, Error, grpc::stream::minecraft_bridge};
use poise::serenity_prelude::{self as serenity};
use tracing::info;

/// Handles the Ready event when the bot successfully connects
pub async fn handle(
    _ctx: &serenity::Context,
    data: &Data,
    guild_id: &serenity::GuildId,
    user: &serenity::User,
    _member_data_if_available: &Option<serenity::Member>,
) -> Result<(), Error> {
    info!(
        "[guild_member_removal::handle] User {} left guild {}",
        user.tag(),
        guild_id
    );

    let user_id = user.id.get() as i64;

    match sqlx::query!(
        "SELECT player_name, player_ipv4 FROM minecraft_users WHERE discord_user_id = $1",
        user_id
    )
    .fetch_all(&data.db)
    .await
    {
        Ok(records) => {
            for record in records {
                let player_name = record.player_name;
                let player_ipv4 = record.player_ipv4;

                info!(
                    "[guild_member_removal::handle] Found linked Minecraft user: {} (IP: {})",
                    player_name, player_ipv4
                );

                // Send PlayerDisconnect event to Minecraft server via gRPC
                minecraft_bridge::disconnect::guild_member_removal(
                    Arc::new(data.clone()),
                    player_name,
                    player_ipv4,
                )
                .await;
            }

            sqlx::query!("DELETE FROM discord_users WHERE id = $1", user_id)
                .execute(&data.db)
                .await?;
        }
        _ => {
            info!(
                "[guild_member_removal::handle] No linked Minecraft user found for Discord user {}",
                user.tag()
            );
        }
    }

    Ok(())
}
