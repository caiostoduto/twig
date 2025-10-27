pub mod ready;

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
            ready::handle(ctx, data_about_bot, data).await?;
        }
        _ => {}
    }

    Ok(())
}
