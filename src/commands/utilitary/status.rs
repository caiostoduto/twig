use crate::{
    Context, Error,
    utils::{config, docker, embed},
};

use poise::CreateReply;
use std::time::Duration;
use sysinfo::System;

/// Display the bot's current status
#[poise::command(slash_command, category = "Utilitary")]
pub async fn status(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    // Gather statuses concurrently
    let (docker_status, shard_info, tailscale_status) = tokio::join!(
        get_docker_status(),
        get_shard_info(&ctx),
        get_tailscale_status()
    );

    // Initialize system and refresh CPU/Memory
    let mut sys = System::new();
    sys.refresh_cpu_usage();
    sys.refresh_memory();

    // Calculate CPU usage
    let cpu_usage = if !sys.cpus().is_empty() {
        sys.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() / sys.cpus().len() as f32
    } else {
        0.0
    };

    // Create embed response
    let embed = embed::get_embed_template(embed::EmbedStatus::Success)
        .title("📊  Status")
        .fields(vec![
            (
                "#️⃣ Shard Info",
                &format!("{}/{}", shard_info.shard_id, shard_info.shard_count),
                true,
            ),
            (
                "🐕‍🦺 Guilds",
                &format!("{}", ctx.cache().guilds().len()),
                true,
            ),
            (
                "🕒 Uptime",
                &format!(
                    "{}w {}d {}h {}m {}s",
                    config::get_config().start_time.elapsed().as_secs() / 604800,
                    (config::get_config().start_time.elapsed().as_secs() % 604800) / 86400,
                    (config::get_config().start_time.elapsed().as_secs() % 86400) / 3600,
                    (config::get_config().start_time.elapsed().as_secs() % 3600) / 60,
                    config::get_config().start_time.elapsed().as_secs() % 60
                ),
                true,
            ),
            ("⏱️ CPU Usage", &format!("{:.2}%", cpu_usage), true),
            (
                "📈 Memory Usage",
                &format!(
                    "{:.2}/{:.2}GB",
                    sys.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0,
                    sys.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0
                ),
                true,
            ),
            ("\u{200b}", &"\u{200b}".to_string(), true),
            ("<:docker:1431626218800808026> Docker", &docker_status, true),
            (
                "<:tailscale:1431362623194267809> Tailscale",
                &tailscale_status,
                true,
            ),
            ("\u{200b}", &"\u{200b}".to_string(), true),
        ]);

    // Send the response
    ctx.send(CreateReply::default().embed(embed).ephemeral(true))
        .await?;

    Ok(())
}

/// Check Tailscale status
async fn get_tailscale_status() -> String {
    // Try to reach the Tailscale local IP
    match reqwest::Client::builder()
        .timeout(Duration::from_millis(200))
        .build()
        .unwrap()
        .get("http://100.100.100.100")
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                "Running".to_string()
            } else {
                "Not running".to_string()
            }
        }
        Err(_) => "Not running".to_string(),
    }
}

async fn get_docker_status() -> String {
    // Check if Docker socket is configured
    if config::get_config().docker_socket.is_some() {
        // Create a client to connect to the Docker socket
        let client = docker::DockerClient::new();

        match client.ping().await {
            Ok(response) => {
                if response.status().is_success() {
                    return "Running".to_string();
                } else {
                    return "Not running".to_string();
                }
            }
            Err(_) => {
                return "Not configured".to_string();
            }
        }
    } else {
        return "Not configured".to_string();
    }
}

struct ShardInfo {
    shard_id: u32,
    shard_count: usize,
}

/// Gather shard information
async fn get_shard_info(ctx: &Context<'_>) -> ShardInfo {
    // Shard IDs are zero-indexed, so we add 1 for display purposes
    let shard_id = ctx.serenity_context().shard_id.get() + 1;
    let shard_count = ctx
        .framework()
        .shard_manager()
        .shards_instantiated()
        .await
        .len();

    ShardInfo {
        shard_id,
        shard_count,
    }
}
