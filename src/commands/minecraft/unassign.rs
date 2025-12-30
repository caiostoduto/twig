use poise::CreateReply;

use crate::{
    Context, Error,
    commands::minecraft::autocomplete_assigned_server,
    utils::{checks, embed},
};

/// Unassign a Discord role from a Minecraft server
#[poise::command(slash_command, guild_only = true, check = "checks::is_owner")]
pub async fn unassign(
    ctx: Context<'_>,

    #[description = "Server to unassign the role from"]
    #[autocomplete = "autocomplete_assigned_server"]
    server: String,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    // Check if server exists and belongs to guild
    let guild_id = u64::from(ctx.guild_id().unwrap()) as i64;
    let server_result = sqlx::query!(
        "SELECT minecraft_servers.id FROM minecraft_servers
        JOIN minecraft_proxies ON minecraft_servers.proxy_id = minecraft_proxies.id
        WHERE
            minecraft_proxies.discord_guild_id = ?1 AND
            minecraft_servers.server_type IS NOT NULL AND
            minecraft_servers.server_name = ?2",
        guild_id,
        server
    )
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

    let server_id = server_result.unwrap().id;

    // Update server discord_role_id
    sqlx::query!(
        "UPDATE minecraft_servers SET server_type = NULL, discord_role_id = NULL
         WHERE id = ?1",
        server_id
    )
    .execute(&ctx.data().db)
    .await?;

    let embed = embed::success()
        .title("<:minecraft:1435794853517721722>  Minecraft unassign server")
        .description("Role successfully unassigned from the specified server.");

    ctx.send(CreateReply::default().embed(embed).ephemeral(true))
        .await?;

    Ok(())
}
