use poise::{
    CreateReply,
    serenity_prelude::{Mentionable, RoleId, all::Role},
};
use tracing::info;

use crate::{
    Context, Error,
    utils::{checks, config, embed},
};

/// Tailscale role management commands
#[poise::command(
    slash_command,
    subcommands("assign", "unassign", "list"),
    check = "checks::is_owner"
)]
pub async fn role(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Autocomplete Tailscale tags
async fn autocomplete_tags(ctx: Context<'_>, _partial: &str) -> Vec<String> {
    let tags = fetch_tags_from_tailscale_api(&ctx).await;
    sync_tags_with_database(&ctx, &tags);

    info!("[autocomplete_tags] ({}): {:?}", tags.len(), tags);
    tags
}

/// Assign a role to a  Tailscale tag
#[poise::command(slash_command, guild_only = true)]
async fn assign(
    ctx: Context<'_>,

    #[description = "Role to be assigned"] role: Role,

    #[description = "Tailscale tag to assign"]
    #[autocomplete = "autocomplete_tags"]
    tag: String,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    // Check if role is @everyone
    if role.id.get() == role.guild_id.get() {
        let embed = embed::get_embed_template(embed::EmbedStatus::Error)
            .title("<:tailscale:1431362623194267809>  Tailscale role assign")
            .description("You cannot assign the @everyone role.");

        ctx.send(CreateReply::default().embed(embed).ephemeral(true))
            .await?;

        return Ok(());
    }

    // Check if tag exists
    if !tailscale_tag_exists(&ctx, &tag).await {
        let embed = embed::get_embed_template(embed::EmbedStatus::Error)
            .title("<:tailscale:1431362623194267809>  Tailscale role assign")
            .description(format!("Tag `{}` does not exist.", tag));

        ctx.send(CreateReply::default().embed(embed).ephemeral(true))
            .await?;

        return Ok(());
    }

    // Assign role to tag in database
    tailscale_assign_tag(&ctx, &tag, &role).await;

    let embed = embed::get_embed_template(embed::EmbedStatus::Success)
        .title("<:tailscale:1431362623194267809>  Tailscale role assign")
        .description(format!(
            "Role {} has been assigned to `{}`.",
            role.mention(),
            tag
        ));

    ctx.send(CreateReply::default().embed(embed).ephemeral(true))
        .await?;

    Ok(())
}

/// Unassign a role to a  Tailscale tag
#[poise::command(slash_command, guild_only = true)]
async fn unassign(
    ctx: Context<'_>,

    #[description = "Role to be unassigned"] role: Role,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    // Check if role is @everyone
    if role.id.get() == role.guild_id.get() {
        let embed = embed::get_embed_template(embed::EmbedStatus::Error)
            .title("<:tailscale:1431362623194267809>  Tailscale role unassign")
            .description("You cannot unassign the @everyone role.");

        ctx.send(CreateReply::default().embed(embed).ephemeral(true))
            .await?;

        return Ok(());
    }

    // Remove role assignment from database
    let mut deleted_role = false;
    if let Ok(db) = ctx.data().db.lock() {
        match db.execute(
            "DELETE FROM discord_guild_roles WHERE id = ?1",
            [role.id.get()],
        ) {
            Ok(updated) => deleted_role = updated > 0,
            Err(_) => {}
        }
    }

    let embed = embed::get_embed_template(embed::EmbedStatus::Success)
        .title("<:tailscale:1431362623194267809>  Tailscale role unassign")
        .description(if deleted_role {
            format!("Role {} has been unassigned.", role.mention())
        } else {
            format!(
                "Role {} was not assigned to any tailscale tag.",
                role.mention()
            )
        });

    ctx.send(CreateReply::default().embed(embed).ephemeral(true))
        .await?;

    Ok(())
}

/// List all role assignments
#[poise::command(slash_command)]
async fn list(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let mut embed = embed::get_embed_template(embed::EmbedStatus::Success)
        .title("<:tailscale:1431362623194267809>  Tailscale role list");

    if let Ok(db) = ctx.data().db.lock() {
        let mut stmt = db
            .prepare(
                "SELECT discord_guild_roles.discord_guild_id, discord_guild_roles.id, discord_guild_roles.tailscale_tag_id
             FROM discord_guild_roles
             ORDER BY discord_guild_roles.discord_guild_id",
            )
            .unwrap();

        let role_iter = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, u64>(0)?,    // guild_id
                    row.get::<_, u64>(1)?,    // role_id
                    row.get::<_, String>(2)?, // tag_id
                ))
            })
            .unwrap();

        // Group roles by guild_id
        let mut guilds: std::collections::HashMap<u64, Vec<(u64, String)>> =
            std::collections::HashMap::new();

        for role_result in role_iter {
            let (guild_id, role_id, tag_id) = role_result?;
            guilds
                .entry(guild_id)
                .or_insert_with(Vec::new)
                .push((role_id, tag_id));
        }

        if guilds.is_empty() {
            embed = embed.description("No role assignments found.");
        } else {
            // Add a field for each guild
            for (guild_id, roles) in guilds {
                let mut field_value = String::new();
                for (role_id, tag_id) in roles {
                    if ctx.guild_id().is_some() && guild_id == ctx.guild_id().unwrap().get() {
                        field_value.push_str(&format!(
                            "<@&{}> \\<@&{}> → `{}`\n",
                            role_id, role_id, tag_id
                        ));
                    } else {
                        match ctx.cache().guild(guild_id) {
                            Some(guild) => {
                                field_value.push_str(&format!(
                                    "{}\\<@&{}> → `{}`\n",
                                    match guild.roles.get(&RoleId::new(role_id)) {
                                        Some(role) => format!("{} ", role.name),
                                        None => "".to_string(),
                                    },
                                    role_id,
                                    tag_id
                                ));
                            }
                            None => {
                                field_value
                                    .push_str(&format!("\\<@&{}> → `{}`\n", role_id, tag_id));
                            }
                        }
                    }
                }

                match ctx.cache().guild(guild_id) {
                    Some(guild) => {
                        embed = embed.field(
                            format!("{} ({})", guild.name, guild.id),
                            field_value,
                            false,
                        );
                    }
                    None => {
                        embed = embed.field(format!("({})", guild_id), field_value, false);
                    }
                }
            }
        }
    }

    ctx.send(CreateReply::default().embed(embed).ephemeral(true))
        .await?;

    Ok(())
}

async fn tailscale_tag_exists(ctx: &Context<'_>, tag: &str) -> bool {
    let mut tag_exists = false;

    // Check if exists in database
    if let Ok(db) = ctx.data().db.lock() {
        tag_exists = db
            .execute(
                "SELECT EXISTS (SELECT 1 FROM tailscale_tags WHERE id = ?1)",
                [tag],
            )
            .ok()
            .is_some();
    }

    if tag_exists {
        return true;
    } else {
        // Fetch tags from Tailscale API
        let tags = fetch_tags_from_tailscale_api(ctx).await;
        sync_tags_with_database(ctx, &tags);

        return tags.contains(&tag.to_string());
    }
}

/// Assign tag to the database
async fn tailscale_assign_tag(ctx: &Context<'_>, tag: &str, role: &Role) {
    if let Ok(db) = ctx.data().db.lock() {
        // Remove any existing assignment for this tag
        db.execute(
            "DELETE FROM discord_guild_roles WHERE tailscale_tag_id = ?1",
            [tag],
        )
        .ok();

        // Ensure guild exists in database
        db.execute(
            "INSERT OR IGNORE INTO discord_guilds (id) VALUES (?1)",
            (role.guild_id.get(),),
        )
        .ok();

        // Assign role to tag
        db.execute(
            "INSERT OR REPLACE INTO discord_guild_roles (id, discord_guild_id, tailscale_tag_id) VALUES (?1, ?2, ?3)",
            (role.id.get(), role.guild_id.get(), tag),
        ).ok();
    }
}

/// Fetch tags from Tailscale API
async fn fetch_tags_from_tailscale_api(ctx: &Context<'_>) -> Vec<String> {
    // Fetch tags from Tailscale API
    let client = &ctx.data().tailscale_client;

    // Fetch policy file
    match client.get_policy_file().await {
        Ok(policy_file) => {
            // Extract tags
            let Some(tag_owners) = policy_file.get("tagOwners").and_then(|v| v.as_object()) else {
                return Vec::new();
            };

            // Collect tags that include the configured tag owner
            let tags = tag_owners
                .iter()
                .filter_map(|(key, owners)| {
                    let includes = owners.as_array().map_or(false, |arr| {
                        arr.iter()
                            .any(|s| s.as_str() == config::get_config().tailscale_tag.as_deref())
                    });

                    if includes { Some(key.clone()) } else { None }
                })
                .collect();

            return tags;
        }
        Err(_err) => {
            return Vec::new();
        }
    }
}

/// Synchronize tags with the database
fn sync_tags_with_database(ctx: &Context<'_>, tags: &[String]) {
    if let Ok(db) = ctx.data().db.lock() {
        // Remove tags that are no longer present
        if !tags.is_empty() {
            // Create placeholders for the IN clause
            let placeholders = std::iter::repeat("?")
                .take(tags.len())
                .collect::<Vec<_>>()
                .join(",");

            // Prepare the DELETE statement
            let mut stmt = db
                .prepare(&format!(
                    "DELETE FROM tailscale_tags WHERE id NOT IN ({})",
                    placeholders
                ))
                .ok()
                .unwrap();

            // Bind all tag values as parameters
            let params: Vec<&dyn rusqlite::ToSql> =
                tags.iter().map(|t| t as &dyn rusqlite::ToSql).collect();
            stmt.execute(rusqlite::params_from_iter(params.iter())).ok();
        } else {
            // If no tags, delete all
            db.execute("DELETE FROM tailscale_tags", []).ok();
        }

        // Insert new tags
        for tag in tags {
            db.execute(
                "INSERT OR IGNORE INTO tailscale_tags (id) VALUES (?)",
                [tag],
            )
            .ok();
        }
    }
}
