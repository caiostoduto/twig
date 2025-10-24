use poise::CreateReply;
use tokio::time::Instant;

use crate::{Context, Error, utils::embed};

/// Check the bot's latency and connection status
#[poise::command(slash_command, category = "Utilitary")]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    // Measure the time taken to defer the response
    let api_latency = get_latency_api(&ctx).await?;
    // Gather shard information
    let (shard_id, shard_count) = get_shard_info(&ctx).await;
    // Get gateway ping
    let gateway_ping = ctx.ping().await.as_millis();

    // Create embed response
    let embed = embed::get_embed_template(embed::EmbedStatus::Success)
        .title("🏓  Pong!")
        .fields(vec![
            ("#️⃣ Shard", &format!("{}/{}", shard_id, shard_count), true),
            ("📬 API Latency", &format!("{:.2}ms", api_latency), true),
            ("⛩️ Gateway", &format!("{:.2}ms", gateway_ping), true),
        ]);

    // Send the response
    ctx.send(CreateReply::default().embed(embed).ephemeral(true))
        .await?;

    Ok(())
}

/// Measure the time taken to defer the response
async fn get_latency_api(ctx: &Context<'_>) -> Result<u64, Error> {
    let start = Instant::now();
    ctx.defer_ephemeral().await?;
    let duration = start.elapsed();

    Ok(duration.as_millis() as u64)
}

/// Gather shard information
async fn get_shard_info(ctx: &Context<'_>) -> (u32, usize) {
    let shard_id = ctx.serenity_context().shard_id.get() + 1;
    let shard_count = ctx
        .framework()
        .shard_manager()
        .shards_instantiated()
        .await
        .len();

    (shard_id, shard_count)
}
