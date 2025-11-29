use crate::{
    Context, Error,
    utils::{config, docker, embed},
};

use poise::CreateReply;
use sysinfo::System;

/// Time constants for uptime formatting
const SECS_PER_WEEK: u64 = 604800;
const SECS_PER_DAY: u64 = 86400;
const SECS_PER_HOUR: u64 = 3600;
const SECS_PER_MINUTE: u64 = 60;

/// Display the bot's current status
#[poise::command(slash_command, category = "Utilitary")]
pub async fn status(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    // Gather statuses concurrently
    let (docker_status, shard_info) = tokio::join!(get_docker_status(), get_shard_info(&ctx));

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
    let embed = embed::success().title("📊  Status").fields(vec![
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
            &format_uptime(config::get_config().start_time.elapsed().as_secs()),
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
        ("\u{200b}", &"\u{200b}".to_string(), true),
    ]);

    // Send the response
    ctx.send(CreateReply::default().embed(embed).ephemeral(true))
        .await?;

    Ok(())
}

/// Formats uptime in seconds to a human-readable string
///
/// # Arguments
/// * `total_secs` - Total uptime in seconds
///
/// # Returns
/// A formatted string in the format "Xw Xd Xh Xm Xs"
fn format_uptime(total_secs: u64) -> String {
    let weeks = total_secs / SECS_PER_WEEK;
    let days = (total_secs % SECS_PER_WEEK) / SECS_PER_DAY;
    let hours = (total_secs % SECS_PER_DAY) / SECS_PER_HOUR;
    let minutes = (total_secs % SECS_PER_HOUR) / SECS_PER_MINUTE;
    let seconds = total_secs % SECS_PER_MINUTE;

    format!("{}w {}d {}h {}m {}s", weeks, days, hours, minutes, seconds)
}

/// Check Docker status
async fn get_docker_status() -> String {
    // Check if Docker socket is configured
    let Some(_) = config::get_config().docker_socket.as_ref() else {
        return "Not configured".to_string();
    };

    // Create a client to connect to the Docker socket
    let client = docker::DockerClient::new();

    match client.ping().await {
        Ok(response) if response.status().is_success() => "Running".to_string(),
        _ => "Not running".to_string(),
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
