mod guild_member_removal;
mod guild_member_update;
mod ready;

use crate::{Data, Error};
use poise::serenity_prelude::{self as serenity};
use tracing::debug;

/// Main event handler that routes events to specific handlers
pub async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    debug!(
        "[event_handler] Received event: {:?}",
        event.snake_case_name()
    );

    match event {
        serenity::FullEvent::Ready { data_about_bot } => {
            ready::handle(ctx, data, data_about_bot).await?;
        }
        serenity::FullEvent::GuildMemberRemoval {
            guild_id,
            user,
            member_data_if_available,
        } => {
            guild_member_removal::handle(ctx, data, guild_id, user, member_data_if_available)
                .await?;
        }
        serenity::FullEvent::GuildMemberUpdate {
            old_if_available,
            new,
            event,
        } => {
            guild_member_update::handle(ctx, data, old_if_available, new, event).await?;
        }
        _ => {}
    }

    Ok(())
}
