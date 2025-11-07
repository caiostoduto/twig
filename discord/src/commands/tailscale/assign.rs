use poise::CreateReply;
use serde::de::Error as SerdeDeError;
use std::collections::HashSet;
use tracing::info;

use crate::{
    Context, Error,
    utils::{checks, config, embed, tailscale::TailscaleError},
};

async fn autocomplete_tags(ctx: Context<'_>, partial: &str) -> Vec<String> {
    let tags: Vec<String> = fetch_available_tags(&ctx)
        .await
        .into_iter()
        .filter(|tag| tag.contains(partial))
        .collect();

    tags
}

/// Assign a Tailscale tag to the guild
#[poise::command(slash_command, guild_only = true, check = "checks::is_owner")]
pub async fn assign(
    ctx: Context<'_>,

    #[description = "Tailscale tag to assign to this guild"]
    #[autocomplete = "autocomplete_tags"]
    tag: String,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    // Fetch tags from Tailscale API and sync with database
    let tags = fetch_available_tags(&ctx).await;

    // Check if tag exists
    if !tags.contains(&tag) {
        let embed = embed::get_embed_template(embed::EmbedStatus::Error)
            .title("<:tailscale:1431362623194267809>  Tailscale assign")
            .description("The specified tailscale tag does not exist or isn't available.");

        ctx.send(CreateReply::default().embed(embed).ephemeral(true))
            .await?;

        return Ok(());
    }

    // Insert tailscale tag if not exists
    sqlx::query("INSERT INTO tailscale_tags (id) VALUES (?1)")
        .bind(&tag)
        .execute(&ctx.data().db)
        .await?;

    // Nullify existing assignments for guilds with this tailscale tag
    sqlx::query("UPDATE discord_guilds SET tailscale_tag_id = NULL WHERE tailscale_tag_id = ?1")
        .bind(&tag)
        .execute(&ctx.data().db)
        .await?;

    // Insert or replace guild with tailscale tag assignment
    sqlx::query("INSERT OR REPLACE INTO discord_guilds (id, tailscale_tag_id) VALUES (?1, ?2)")
        .bind(u64::from(ctx.guild_id().unwrap()) as i64)
        .bind(&tag)
        .execute(&ctx.data().db)
        .await?;

    let embed = embed::get_embed_template(embed::EmbedStatus::Success)
        .title("<:tailscale:1431362623194267809>  Tailscale assign")
        .description("Guild successfully assigned to the Tailscale tag.");

    ctx.send(CreateReply::default().embed(embed).ephemeral(true))
        .await?;

    Ok(())
}

/// Fetch tags from Tailscale API
async fn fetch_tags_from_tailscale_api(ctx: &Context<'_>) -> Result<Vec<String>, TailscaleError> {
    // Fetch tags from Tailscale API
    let ts_client = &ctx.data().ts_client;

    // Fetch policy file
    let policy_file = ts_client.get_policy_file().await?;

    // Extract tags
    let Some(tag_owners) = policy_file.get("tagOwners").and_then(|v| v.as_object()) else {
        return Err(TailscaleError::Json(serde_json::Error::custom("msg")));
    };

    // Collect tags that include the configured tag owner
    let tags: Vec<String> = tag_owners
        .iter()
        .filter_map(|(key, owners)| {
            let includes = owners.as_array().map_or(false, |arr| {
                arr.iter()
                    .any(|s| s.as_str() == config::get_config().tailscale_tag.as_deref())
            });

            if includes { Some(key.clone()) } else { None }
        })
        .collect();

    Ok(tags)
}

async fn fetch_available_tags(ctx: &Context<'_>) -> Vec<String> {
    // Fetch tags from Tailscale API
    let tags = match fetch_tags_from_tailscale_api(&ctx).await {
        Ok(tags) => {
            info!("[fetch_available_tags] ({}): {:?}", tags.len(), tags);
            tags
        }
        Err(err) => {
            tracing::error!(
                "[fetch_available_tags] Failed to fetch tags from Tailscale API: {}",
                err
            );

            Vec::new()
        }
    };

    // Query which tags already exist
    let placeholders = tags.iter().map(|_| "?").collect::<Vec<_>>().join(",");

    let query_str = format!(
        "SELECT id FROM tailscale_tags WHERE id IN ({})",
        placeholders
    );
    let mut query = sqlx::query_scalar::<_, String>(&query_str);

    for tag in &tags {
        query = query.bind(tag);
    }

    let existing_tags: HashSet<String> = query
        .fetch_all(&ctx.data().db)
        .await
        .unwrap()
        .into_iter()
        .collect();

    // Get tags not in DB
    tags.into_iter()
        .filter(|tag| !existing_tags.contains(tag))
        .collect()
}
