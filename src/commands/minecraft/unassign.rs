use poise::CreateReply;
use tracing::info;

use crate::{
    Context, Error,
    utils::{checks, embed},
};

async fn autocomplete_server(ctx: Context<'_>, partial: &str) -> Vec<String> {
    let mut server_ids = Vec::new();

    let guild_id = match ctx.guild_id() {
        Some(id) => u64::from(id),
        None => return server_ids,
    };

    match sqlx::query_as::<_, (String,)>(
        "SELECT minecraft_servers.server_name 
        FROM minecraft_servers 
        JOIN minecraft_proxies ON minecraft_servers.proxy_id = minecraft_proxies.id 
        WHERE
            minecraft_proxies.discord_guild_id = ?1 AND
            minecraft_servers.server_type IS NOT NULL AND
            minecraft_servers.server_name LIKE ?2",
    )
    .bind(guild_id as i64)
    .bind(format!("%{}%", partial))
    .fetch_all(&ctx.data().db)
    .await
    {
        Ok(rows) => {
            for (server_name,) in rows {
                server_ids.push(server_name);
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

/// Unassign a Discord role from a Minecraft server
#[poise::command(slash_command, guild_only = true, check = "checks::is_owner")]
pub async fn unassign(
    ctx: Context<'_>,

    #[description = "Server to unassign the role from"]
    #[autocomplete = "autocomplete_server"]
    server: String,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    // Check if server exists and belongs to guild
    let server_result = sqlx::query_as::<_, (i64,)>(
        "SELECT minecraft_servers.id FROM minecraft_servers
        JOIN minecraft_proxies ON minecraft_servers.proxy_id = minecraft_proxies.id
        WHERE
            minecraft_proxies.discord_guild_id = ?1 AND
            minecraft_servers.server_type IS NOT NULL AND
            minecraft_servers.server_name = ?2",
    )
    .bind(u64::from(ctx.guild_id().unwrap()) as i64)
    .bind(&server)
    .fetch_optional(&ctx.data().db)
    .await?;

    if server_result.is_none() {
        let embed = embed::warn()
            .title("<:minecraft:1435794853517721722>  /minecraft unassign server")
            .description("The specified server does not exist at this guild.");

        ctx.send(CreateReply::default().embed(embed).ephemeral(true))
            .await?;

        return Ok(());
    }

    let (server_id,) = server_result.unwrap();

    // Update server discord_role_id
    sqlx::query(
        "UPDATE minecraft_servers SET server_type = NULL, discord_role_id = NULL
         WHERE id = ?1",
    )
    .bind(&server_id.to_string())
    .execute(&ctx.data().db)
    .await?;

    let embed = embed::success()
        .title("<:minecraft:1435794853517721722>  Minecraft unassign server")
        .description("Role successfully unassigned from the specified server.");

    ctx.send(CreateReply::default().embed(embed).ephemeral(true))
        .await?;

    Ok(())
}
