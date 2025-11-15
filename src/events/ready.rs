use crate::{Data, Error};
use poise::serenity_prelude::{self as serenity, ActivityData};
use tracing::info;

/// Handles the Ready event when the bot successfully connects
pub async fn handle(
    ctx: &serenity::Context,
    _data_about_bot: &serenity::Ready,
    _data: &Data,
) -> Result<(), Error> {
    let activity = ActivityData::playing("with Tailscale APIs");
    let status = serenity::OnlineStatus::DoNotDisturb;

    ctx.set_presence(Some(activity), status);

    info!("[ready::handle] Presence set: Playing with Tailscale APIs (Do Not Disturb)");

    Ok(())
}
