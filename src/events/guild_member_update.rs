use std::sync::Arc;

use poise::serenity_prelude as serenity;
use tracing::info;

use crate::{Data, Error};

pub async fn handle(
    _ctx: &serenity::Context,
    data: &Data,
    _old_if_available: &Option<serenity::Member>,
    _new: &Option<serenity::Member>,
    event: &serenity::GuildMemberUpdateEvent,
) -> Result<(), Error> {
    // Check if user is registered in DB
    info!(
        "[guild_member_update::handle] Guild member updated: {} in guild {}",
        event.user.tag(),
        event.guild_id
    );

    let user_id = event.user.id.get() as i64;

    if let Ok(records) = sqlx::query!(
        "SELECT player_name, player_ipv4 FROM minecraft_users WHERE discord_user_id = $1",
        user_id
    )
    .fetch_all(&data.db)
    .await
    {
        info!(
            "[guild_member_update::handle] Found {} linked Minecraft account(s) for Discord user ID {}",
            records.len(),
            user_id
        );

        for record in records {
            let (player_name, player_ipv4) = (record.player_name, record.player_ipv4);

            crate::grpc::stream::disconnect::guild_member_removal(
                Arc::new(data.clone()),
                player_name,
                player_ipv4,
            )
            .await;
        }
    };

    Ok(())
}
