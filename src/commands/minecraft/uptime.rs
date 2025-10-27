use std::collections::HashMap;

use poise::CreateReply;

use crate::{
    Context, Error,
    utils::{config, embed, influxdb},
};

/// Get the uptime of the Minecraft servers
#[poise::command(slash_command)]
pub async fn uptime(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    // Get Minecraft servers uptime data
    let uptimes = get_minecraft_servers_uptime().await;

    // Prepare fields for embed
    let mut fields = uptimes
        .iter()
        .map(|(id, uptime)| {
            let uptime_text = uptime
                .values
                .iter()
                .map(|v| match *v >= 0.5 {
                    true => "<:uptime:1432121768436433018>",
                    false => "<:downtime:1432121789454221492>",
                })
                .collect::<Vec<&str>>()
                .join("");

            (
                format!("{} ({:.2}%)", &id, uptime.mean * 100.0),
                uptime_text,
                false,
            )
        })
        .collect::<Vec<(String, String, bool)>>();

    // Sort fields alphabetically by server name
    fields.sort_by_key(|f| f.0.clone());

    // Create embed response
    let embed = embed::get_embed_template(embed::EmbedStatus::Success)
        .title("📊  Minecraft Status (6h)")
        .fields(fields);

    // Send the response
    ctx.send(CreateReply::default().embed(embed).ephemeral(true))
        .await?;

    Ok(())
}

#[derive(Debug)]
pub struct MinecraftUptime {
    values: Vec<f64>,
    mean: f64,
}

/// Retrieves the health status of all Minecraft server containers
async fn get_minecraft_servers_uptime() -> HashMap<String, MinecraftUptime> {
    let client = influxdb::InfluxDB::new();
    let mut uptimes = HashMap::new();

    for uptime in client
        .unwrap()
        .query(format!(
            "from(bucket: \"{}\")
        |> range(start: -6h)
        |> filter(fn: (r) => r._measurement == \"minecraft_status\")
        |> map(fn: (r) => ({{ r with _value: if r.status == \"success\" then 1.0 else 0.0 }}))
        |> group(columns: [\"host\"])
        |> aggregateWindow(every: 30m, fn: mean, createEmpty: true)
        |> fill(column: \"_value\", value: 0.0)
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
            .entry(uptime[3].clone())
            .or_insert_with(|| MinecraftUptime {
                values: Vec::new(),
                mean: 0.0,
            })
            .values
            .push(uptime[4].parse::<f64>().unwrap());
    }

    for (_host, uptime) in uptimes.iter_mut() {
        let sum: f64 = uptime.values.iter().sum();
        uptime.mean = sum / (uptime.values.len() as f64);
    }

    uptimes
}
