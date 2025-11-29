use std::collections::HashMap;

use chrono::Timelike;
use poise::{CreateReply, serenity_prelude::CreateEmbed};

use crate::{
    Context, Error,
    utils::{config, embed, influxdb},
};

/// Get the uptime of the Minecraft servers
#[poise::command(slash_command)]
pub async fn uptime(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    // Send the response
    ctx.send(
        CreateReply::default()
            .embed(embed_message().await)
            .ephemeral(true),
    )
    .await?;

    Ok(())
}

#[derive(Debug)]
pub struct MinecraftUptime {
    values: Vec<f64>,
    mean: f64,
}

async fn embed_message() -> CreateEmbed {
    // Get Minecraft servers uptime data
    let uptimes = get_minecraft_servers_uptime().await;

    // Prepare fields for embed
    let mut fields = uptimes
        .iter()
        .map(|(id, uptime)| {
            let uptime_text = uptime
                .values
                .iter()
                .map(|v| match *v >= 0.8 {
                    true => "<:uptime:1432121768436433018>",
                    false => "<:downtime:1432121789454221492>",
                })
                .collect::<Vec<&str>>()
                .join("");

            (
                format!("{} ({:.4}%)", &id, uptime.mean * 100.0),
                uptime_text,
                false,
            )
        })
        .collect::<Vec<(String, String, bool)>>();

    // Sort fields alphabetically by server name
    fields.sort_by_key(|f| f.0.clone());

    // Create embed response
    return embed::success()
        .title("📊  Minecraft Status (6h)")
        .fields(fields);
}

/// Retrieves the health status of all Minecraft server containers
async fn get_minecraft_servers_uptime() -> HashMap<String, MinecraftUptime> {
    let client = influxdb::InfluxDB::new().unwrap();
    let mut uptimes = HashMap::new();

    // Calculate offset to align 30m windows with current time
    let now = chrono::Utc::now();
    let offset_minutes = now.minute() % 30;
    let offset = format!("{}m", offset_minutes);

    // Get aggregated historical data (6h ago to 1m ago, aligned to current time)
    for uptime in client
        .query(format!(
            "from(bucket: \"{}\")
        |> range(start: -6h, stop: -1m)
        |> filter(fn: (r) => r._measurement == \"minecraft_status\")
        |> map(fn: (r) => ({{ r with _value: if r.status == \"success\" then 1.0 else 0.0 }}))
        |> group(columns: [\"host\"])
        |> aggregateWindow(every: 30m, fn: mean, createEmpty: true, offset: {})
        |> fill(column: \"_value\", value: 0.0)
        |> keep(columns: [\"_value\", \"host\"])",
            config::get_config()
                .influxdb_bucket
                .as_deref()
                .expect("INFLUXDB_BUCKET environment variable must be set"),
            offset
        ))
        .await
        .unwrap()
        .into_iter()
    {
        uptimes
            .entry(uptime[3].clone())
            .or_insert_with(|| MinecraftUptime {
                values: Vec::new(),
                mean: 0.0,
            })
            .values
            .push(uptime[4].parse::<f64>().unwrap());
    }

    // Calculate mean uptime for each server
    for (_host, uptime) in uptimes.iter_mut() {
        let sum: f64 = uptime.values.iter().sum();
        uptime.mean = sum / (uptime.values.len() as f64);
    }

    // Get current real-time status (last 1 minute)
    for uptime in client
        .query(format!(
            "from(bucket: \"{}\")
        |> range(start: -1m)
        |> filter(fn: (r) => r._measurement == \"minecraft_status\")
        |> map(fn: (r) => ({{ r with _value: if r.status == \"success\" then 1.0 else 0.0 }}))
        |> group(columns: [\"host\"])
        |> last()
        |> keep(columns: [\"_value\", \"host\"])",
            config::get_config()
                .influxdb_bucket
                .as_deref()
                .expect("INFLUXDB_BUCKET environment variable must be set")
        ))
        .await
        .unwrap()
        .into_iter()
    {
        uptimes
            .entry(uptime[4].clone())
            .or_insert_with(|| MinecraftUptime {
                values: vec![0.0; 12],
                mean: 0.0,
            })
            .values
            .push(uptime[3].parse::<f64>().unwrap());
    }

    uptimes
}
