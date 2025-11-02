use poise::CreateReply;
use reqwest::Client;
use tokio::time::Instant;

use crate::{
    Context, Error,
    utils::{config, embed},
};

/// API endpoint for Tailscale ping check
const TAILSCALE_PING_ENDPOINT: &str = "/ping";

/// Check the bot's latency and connection status
#[poise::command(slash_command, category = "Utilitary")]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    // Gather latencies concurrently
    let (tailscale_latency, discord_latency, gateway_ping) =
        tokio::join!(get_tailscale_latency(), get_discord_latency(&ctx), async {
            ctx.ping().await.as_millis()
        });

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
            (
                "<:tailscale:1431362623194267809> Tailscale (http)",
                &format!("{:.2}ms", tailscale_latency),
                true,
            ),
        ]);

    // Send the response
    ctx.send(CreateReply::default().embed(embed).ephemeral(true))
        .await?;

    Ok(())
}

/// Measure the time taken to defer the response
pub async fn get_discord_latency(ctx: &Context<'_>) -> u128 {
    let start = Instant::now();
    ctx.defer_ephemeral().await.ok();
    start.elapsed().as_millis()
}

/// Measures latency to Tailscale API
pub async fn get_tailscale_latency() -> u128 {
    let client = Client::new();
    let url = format!("{}{}", config::get_config().tailscale_api_base, TAILSCALE_PING_ENDPOINT);

    let start = Instant::now();
    let _ = client.get(url).send().await;
    start.elapsed().as_millis()
}
