use poise::{CreateReply, serenity_prelude::Role};
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

    let guild_id_i64 = guild_id as i64;
    let pattern = format!("%{}%", partial);

    match sqlx::query!(
        "SELECT minecraft_servers.server_name 
        FROM minecraft_servers 
        JOIN minecraft_proxies ON minecraft_servers.proxy_id = minecraft_proxies.id 
        WHERE
            (minecraft_proxies.discord_guild_id = ?1 OR minecraft_proxies.discord_guild_id IS NULL) AND
            minecraft_servers.discord_role_id IS NULL AND
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

/// Assign a Discord role to a Minecraft server
#[poise::command(slash_command, guild_only = true, check = "checks::is_owner")]
pub async fn assign(
    ctx: Context<'_>,

    #[description = "Server to assign the role to"]
    #[autocomplete = "autocomplete_server"]
    server: String,

    #[description = "Role to assign to the server"] role: Role,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    // Check if role is @everyone
    if role.id.get() == role.guild_id.get() {
        let embed = embed::get_embed_template(embed::EmbedStatus::Error)
            .title("<:minecraft:1435794853517721722>  Minecraft assign server")
            .description("You cannot assign the @everyone role.");

        ctx.send(CreateReply::default().embed(embed).ephemeral(true))
            .await?;

        return Ok(());
    }

    // Check if server exists and belongs to guild
    let server_result = sqlx::query_as::<_, (i64, String)>(
        "SELECT minecraft_servers.id, minecraft_servers.proxy_id FROM minecraft_servers
        JOIN minecraft_proxies ON minecraft_servers.proxy_id = minecraft_proxies.id
        WHERE
            (minecraft_proxies.discord_guild_id = ?1 OR minecraft_proxies.discord_guild_id IS NULL) AND
            minecraft_servers.discord_role_id IS NULL AND
            minecraft_servers.server_name = ?2",
    )
    .bind(u64::from(ctx.guild_id().unwrap()) as i64)
    .bind(&server)
    .fetch_optional(&ctx.data().db)
    .await?;

    if server_result.is_none() {
        let embed = embed::get_embed_template(embed::EmbedStatus::Error)
            .title("<:minecraft:1435794853517721722>  /minecraft assign server")
            .description("The specified server doesn't exist or isn't available at this guild.");

        ctx.send(CreateReply::default().embed(embed).ephemeral(true))
            .await?;

        return Ok(());
    }

    let (server_id, proxy_id) = server_result.unwrap();

    // Insert guild if not exists
    let guild_id = u64::from(ctx.guild_id().unwrap()) as i64;
    sqlx::query!(
        "INSERT OR IGNORE INTO discord_guilds (id) VALUES (?1)",
        guild_id
    )
    .execute(&ctx.data().db)
    .await?;

    // Update proxy guild_id
    sqlx::query("UPDATE minecraft_proxies SET discord_guild_id = ?1 WHERE id = ?2")
        .bind(u64::from(ctx.guild_id().unwrap()) as i64)
        .bind(&proxy_id)
        .execute(&ctx.data().db)
        .await?;

    // Update server discord_role_id
    sqlx::query(
        "UPDATE minecraft_servers SET discord_role_id = ?1
         WHERE id = ?2",
    )
    .bind(u64::from(role.id) as i64)
    .bind(&server_id.to_string())
    .execute(&ctx.data().db)
    .await?;

    let embed = embed::get_embed_template(embed::EmbedStatus::Success)
        .title("<:minecraft:1435794853517721722>  Minecraft assign server")
        .description("Role successfully assigned to the specified server.");

    ctx.send(CreateReply::default().embed(embed).ephemeral(true))
        .await?;

    Ok(())
}
