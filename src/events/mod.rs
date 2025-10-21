pub mod guild_member_removal;

use crate::{Data, Error};
use poise::serenity_prelude as serenity;

/// Main event handler that routes events to specific handlers
pub async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::GuildMemberRemoval { guild_id, user, .. } => {
            guild_member_removal::handle(ctx, guild_id, user, data).await?;
        }
        _ => {}
    }

    Ok(())
}
