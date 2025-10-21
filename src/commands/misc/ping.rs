use poise::CreateReply;
use tokio::time::Instant;

use crate::{Context, Error, utils::embed};

/// Check the bot's latency and connection status
#[poise::command(slash_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let start = Instant::now();
    ctx.defer_ephemeral().await?;
    let duration = start.elapsed();

    let shard_id = ctx.serenity_context().shard_id.get() + 1;
    let shard_count = ctx
        .framework()
        .shard_manager()
        .shards_instantiated()
        .await
        .len();
    let api_latency = duration.as_millis();
    let gateway_ping = ctx.ping().await.as_millis();

    let embed = embed::get_embed_template(embed::EmbedStatus::Success)
        .title("🏓  Pong!")
        .fields(vec![
            ("#️⃣ Shard", &format!("{}/{}", shard_id, shard_count), true),
            ("📬 API Latency", &format!("{:.2}ms", api_latency), true),
            ("⛩️ Gateway", &format!("{:.2}ms", gateway_ping), true),
        ]);

    ctx.send(CreateReply::default().embed(embed).ephemeral(true))
        .await?;
    Ok(())
}
