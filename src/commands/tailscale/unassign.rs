use poise::CreateReply;
use tracing::info;

use crate::{
    Context, Error,
    utils::{checks, embed},
};

async fn autocomplete_tag(ctx: Context<'_>, partial: &str) -> Vec<String> {
    let tags: Vec<String> = sqlx::query_as::<_, (String,)>(
        "SELECT id
        FROM tailscale_tags
        WHERE id LIKE ?1",
    )
    .bind(format!("%{}%", partial))
    .fetch_all(&ctx.data().db)
    .await
    .unwrap()
    .into_iter()
    .map(|(tag,)| tag)
    .collect();

    info!("[autocomplete_tag] ({}): {:?}", tags.len(), tags);
    tags
}

/// Unassign a Discord role from a Minecraft server
#[poise::command(slash_command, guild_only = true, check = "checks::is_owner")]
pub async fn unassign(
    ctx: Context<'_>,

    #[description = "Tailscale tag to unassign"]
    #[autocomplete = "autocomplete_tag"]
    mut tag: Option<String>,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    if tag.is_none() {
        tag = sqlx::query_as::<_, (String,)>(
            "SELECT tailscale_tag_id
            FROM discord_guilds
            WHERE id = ?1 LIMIT 1",
        )
        .bind(u64::from(ctx.guild_id().unwrap()) as i64)
        .fetch_optional(&ctx.data().db)
        .await
        .ok()
        .flatten()
        .map(|(tag,)| tag);
    }

    // Remove tailscale tag assignment from guild and check if any row was affected
    let result = sqlx::query(
        "DELETE FROM tailscale_tags
        WHERE id = ?1",
    )
    .bind(&tag)
    .execute(&ctx.data().db)
    .await?;

    if result.rows_affected() == 0 {
        let embed = embed::get_embed_template(embed::EmbedStatus::Error)
            .title("<:tailscale:1431362623194267809>  /tailscale unassign")
            .description("The specified tailscale tag is not assigned to this guild.");

        ctx.send(CreateReply::default().embed(embed).ephemeral(true))
            .await?;

        return Ok(());
    }

    let embed = embed::get_embed_template(embed::EmbedStatus::Success)
        .title("<:tailscale:1431362623194267809>  /tailscale unassign")
        .description("Tailscale tag successfully unassigned from this guild.");

    ctx.send(CreateReply::default().embed(embed).ephemeral(true))
        .await?;

    Ok(())
}
