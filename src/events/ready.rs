use poise::serenity_prelude::{self as serenity, ActivityData};
use tracing::info;

use crate::{Data, Error};

/// Handles the Ready event when the bot successfully connects
pub async fn handle(
    ctx: &serenity::Context,
    _data: &Data,
    _data_about_bot: &serenity::Ready,
) -> Result<(), Error> {
    let activity = ActivityData::playing("with Minecraft APIs");
    let status = serenity::OnlineStatus::DoNotDisturb;

    ctx.set_presence(Some(activity), status);

    info!("[ready::handle] Presence set: Playing with Minecraft APIs (Do Not Disturb)");

    Ok(())
}
