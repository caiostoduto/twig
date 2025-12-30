mod assign;
mod unassign;
mod uptime;

use crate::{Context, Data, Error};

use assign::*;
use tracing::info;
use unassign::*;
use uptime::*;

/// Minecraft command group
#[poise::command(
    slash_command,
    category = "Minecraft",
    subcommands("uptime", "assign", "unassign"),
    subcommand_required = true
)]
pub async fn minecraft(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

pub async fn autocomplete_unassigned_server(ctx: Context<'_>, partial: &str) -> Vec<String> {
    let mut server_ids = Vec::new();

    let guild_id = match ctx.guild_id() {
        Some(id) => u64::from(id),
        None => return server_ids,
    };

    let guild_id_i64 = guild_id as i64;
    let pattern = format!("%{}%", partial);

    match sqlx::query!(
        "SELECT minecraft_servers.server_name 
        FROM minecraft_servers 
        JOIN minecraft_proxies ON minecraft_servers.proxy_id = minecraft_proxies.id 
        WHERE
            (minecraft_proxies.discord_guild_id = ?1 OR minecraft_proxies.discord_guild_id IS NULL) AND
            minecraft_servers.server_name LIKE ?2",
        guild_id_i64,
        pattern
    )
    .fetch_all(&ctx.data().db)
    .await
    {
        Ok(rows) => {
            for row in rows {
                server_ids.push(row.server_name);
            }
        }
        Err(_) => {}
    }

    info!(
        "[autocomplete_server] ({}): {:?}",
        server_ids.len(),
        server_ids
    );

    server_ids
}

async fn autocomplete_assigned_server(ctx: Context<'_>, partial: &str) -> Vec<String> {
    let mut server_ids = Vec::new();

    let guild_id_i64 = match ctx.guild_id() {
        Some(id) => u64::from(id),
        None => return server_ids,
    } as i64;

    let pattern = format!("%{}%", partial);
    match sqlx::query!(
        "SELECT minecraft_servers.server_name 
        FROM minecraft_servers 
        JOIN minecraft_proxies ON minecraft_servers.proxy_id = minecraft_proxies.id 
        WHERE
            minecraft_proxies.discord_guild_id = ?1 AND
            minecraft_servers.server_type IS NOT NULL AND
            minecraft_servers.server_name LIKE ?2",
        guild_id_i64,
        pattern
    )
    .fetch_all(&ctx.data().db)
    .await
    {
        Ok(rows) => {
            for row in rows {
                server_ids.push(row.server_name);
            }
        }
        Err(_) => {}
    }

    info!(
        "[autocomplete_server] ({}): {:?}",
        server_ids.len(),
        server_ids
    );

    server_ids
}

/// Returns all bot commands related to Minecraft category
pub fn commands() -> Vec<poise::Command<Data, Error>> {
    vec![minecraft()]
}
