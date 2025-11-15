use poise::CreateReply;
use tokio::time::Instant;

use crate::{Context, Error, utils::embed};

/// Check the bot's latency and connection status
#[poise::command(slash_command, category = "Utilitary")]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    // Gather latencies concurrently
    let (discord_latency, gateway_ping) =
        tokio::join!(get_discord_latency(&ctx), get_gateway_ping(&ctx));

    // Create embed response
    let embed = embed::get_embed_template(embed::EmbedStatus::Success)
        .title("🏓  Pong!")
        .fields(vec![
            ("⛩️ Gateway", &format!("{:.2}ms", gateway_ping), true),
            (
                "<:discord:1431369538766897334> Discord (defer)",
                &format!("{:.2}ms", discord_latency),
                true,
            ),
        ]);

    // Send the response
    ctx.send(CreateReply::default().embed(embed).ephemeral(true))
        .await?;

    Ok(())
}

/// Measure the time taken to defer the response
async fn get_discord_latency(ctx: &Context<'_>) -> u128 {
    let start = Instant::now();
    ctx.defer_ephemeral().await.ok();
    start.elapsed().as_millis()
}

async fn get_gateway_ping(ctx: &Context<'_>) -> u128 {
    ctx.ping().await.as_millis()
}
