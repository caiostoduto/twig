use poise::serenity_prelude::{self as serenity, ActivityData};
use tracing::{debug, info};

use crate::{Data, Error, utils::minecraft};

/// Handles the Ready event when the bot successfully connects
pub async fn handle(
    ctx: &serenity::Context,
    _data: &Data,
    _data_about_bot: &serenity::Ready,
) -> Result<(), Error> {
    let ctx_clone = ctx.clone();
    tokio::spawn(async move {
        let tracks = minecraft::get_tracks();

        loop {
            for track in &tracks {
                debug!(
                    "[ready::handle] Now playing Minecraft music track: {}",
                    *track
                );

                let activity = ActivityData::listening(format!("{}", track));
                let status = serenity::OnlineStatus::Idle;

                ctx_clone.set_presence(Some(activity), status);

                // Simulate track duration
                tokio::time::sleep(tokio::time::Duration::from_secs(*track.duration_secs)).await;
            }
        }
    });

    info!("[ready::handle] Presence set to Minecraft music tracks.");

    Ok(())
}
