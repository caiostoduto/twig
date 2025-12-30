use poise::{CreateReply, serenity_prelude::Role};
use tracing::info;

use crate::utils::minecraft::MinecraftServerType;
use crate::{
    Context, Error,
    commands::minecraft::autocomplete_unassigned_server,
    utils::{checks, embed},
};

/// Assign a Discord role to a Minecraft server
#[poise::command(slash_command, guild_only = true, check = "checks::is_owner")]
pub async fn assign(
    ctx: Context<'_>,

    #[description = "Server to assign the role to"]
    #[autocomplete = "autocomplete_unassigned_server"]
    server: String,

    #[description = "Role to assign to the server"] role: Option<Role>,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let guild_id = u64::from(ctx.guild_id().unwrap());

    // Check if role is @everyone
    if role.is_some() && role.as_ref().unwrap().id.get() == guild_id {
        let embed = embed::warn()
            .title("<:minecraft:1435794853517721722>  Minecraft assign server")
            .description("You cannot assign the @everyone role.");

        ctx.send(CreateReply::default().embed(embed).ephemeral(true))
            .await?;

        return Ok(());
    }

    // Check if server exists and belongs to guild
    let guild_id_i64 = guild_id as i64;
    let Some((server_id, proxy_id)) = sqlx::query!(
        "SELECT minecraft_servers.id, minecraft_servers.proxy_id FROM minecraft_servers
        JOIN minecraft_proxies ON minecraft_servers.proxy_id = minecraft_proxies.id
        WHERE
            (minecraft_proxies.discord_guild_id = ?1 OR minecraft_proxies.discord_guild_id IS NULL) AND
            minecraft_servers.server_name = ?2",
        guild_id_i64,
        server
    )
    .fetch_optional(&ctx.data().db)
    .await?
    .map(|record| (record.id, record.proxy_id)) else {
        let embed = embed::warn()
            .title("<:minecraft:1435794853517721722>  /minecraft assign server")
            .description("The specified server doesn't exist or isn't available at this guild.");

        ctx.send(CreateReply::default().embed(embed).ephemeral(true))
            .await?;

        return Ok(());
    };

    // Insert guild if not exists
    sqlx::query!(
        "INSERT OR IGNORE INTO discord_guilds (id) VALUES (?1)",
        guild_id_i64
    )
    .execute(&ctx.data().db)
    .await?;

    // Update proxy guild_id
    sqlx::query!(
        "UPDATE minecraft_proxies SET discord_guild_id = ?1 WHERE id = ?2",
        guild_id_i64,
        proxy_id
    )
    .execute(&ctx.data().db)
    .await?;

    // Update server with role
    if role.as_ref().is_some() {
        info!(
            "[minecraft assign] Assigning role {} to server {}",
            role.as_ref().unwrap().id,
            server_id
        );

        let role_id_i64 = u64::from(role.as_ref().unwrap().id) as i64;
        sqlx::query!(
            "UPDATE minecraft_servers SET discord_role_id = ?1, server_type = ?2
         WHERE id = ?3",
            role_id_i64,
            MinecraftServerType::Game as i32,
            server_id
        )
        .execute(&ctx.data().db)
        .await?;

        let embed = embed::success()
            .title("<:minecraft:1435794853517721722>  Minecraft assign server")
            .description("Role successfully assigned to the specified server.");

        ctx.send(CreateReply::default().embed(embed).ephemeral(true))
            .await?;
    } else {
        info!(
            "[minecraft assign] Assigning guild {} to lobby server {}",
            guild_id, server_id
        );

        sqlx::query!(
            "UPDATE minecraft_servers SET server_type = ?1, discord_role_id = NULL
         WHERE id = ?2",
            MinecraftServerType::Lobby as i32,
            server_id
        )
        .execute(&ctx.data().db)
        .await?;

        let embed = embed::success()
            .title("<:minecraft:1435794853517721722>  Minecraft assign server")
            .description("Guild successfully assigned to the specified server.");

        ctx.send(CreateReply::default().embed(embed).ephemeral(true))
            .await?;
    }

    Ok(())
}
