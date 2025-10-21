use poise::{
    CreateReply,
    serenity_prelude::{Mentionable, RoleId, all::Role},
};
use snowflaked::Generator;

use crate::{
    Context, Error,
    utils::{checks, config, embed},
};

#[poise::command(slash_command, subcommands("join", "role"))]
pub async fn tailscale(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Join a  Tailscale network
#[poise::command(slash_command)]
async fn join(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let mut user_id: Option<u64> = None;

    // Firstly, check for permissions
    // any(user.roles for user in guilds where guilds == db.guilds)

    // Secondly, get the user ID from the database
    if let Ok(db) = ctx.data().db.lock() {
        match db
            .query_row(
                "SELECT users.id FROM users
                JOIN discord_users ON users.id = discord_users.user_id
                WHERE discord_users.id = ?1
                LIMIT 1",
                [ctx.author().id.get()],
                |row| row.get(0),
            )
            .ok()
        {
            Some(id) => user_id = Some(id),
            None => {
                user_id = Some(Generator::new(7332).generate());

                db.execute("INSERT INTO users (id) VALUES (?1)", [user_id])
                    .ok();

                db.execute(
                    "INSERT INTO discord_users (id, user_id) VALUES (?1, ?2)",
                    [ctx.author().id.get(), user_id.unwrap()],
                )
                .ok();
            }
        };
    }

    // Delete any existing join links for this user
    // Delete the devices associated with the user

    // Create a new join link

    Ok(())
}

#[poise::command(
    slash_command,
    subcommands("assign", "unassign", "list"),
    check = "checks::is_owner"
)]
async fn role(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

async fn autocomplete_tags(ctx: Context<'_>, _partial: &str) -> Vec<String> {
    let client = &ctx.data().tailscale_client;

    match client.get_policy_file().await {
        Ok(policy_file) => {
            let Some(tag_owners) = policy_file.get("tagOwners").and_then(|v| v.as_object()) else {
                return Vec::new();
            };

            let tags: Vec<String> = tag_owners
                .iter()
                .filter_map(|(key, owners)| {
                    let includes = owners.as_array().map_or(false, |arr| {
                        arr.iter()
                            .any(|s| s.as_str() == Some(&config::get_config().tailscale_tag))
                    });
                    if includes { Some(key.clone()) } else { None }
                })
                .collect();

            // Sync tags with database
            if let Ok(db) = ctx.data().db.lock() {
                // Get existing tags from database
                let existing_tags: Vec<String> = db
                    .prepare("SELECT id FROM tailscale_tags")
                    .and_then(|mut stmt| {
                        stmt.query_map([], |row| row.get(0))
                            .map(|rows| rows.filter_map(|r| r.ok()).collect())
                    })
                    .unwrap_or_default();

                // Delete tags that no longer exist in Tailscale
                for existing_tag in &existing_tags {
                    if !tags.contains(existing_tag) {
                        db.execute("DELETE FROM tailscale_tags WHERE id = ?1", [existing_tag])
                            .ok();
                    }
                }

                // Add new tags that don't exist in database
                for tag in &tags {
                    if !existing_tags.contains(tag) {
                        db.execute(
                            "INSERT OR IGNORE INTO tailscale_tags (id) VALUES (?1)",
                            [tag],
                        )
                        .ok();
                    }
                }
            }

            return tags;
        }
        Err(_err) => {
            return Vec::new();
        }
    }
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
    if role.id.get() == role.guild_id.get() {
        let embed = embed::get_embed_template(embed::EmbedStatus::Error)
            .title("🛜  Tailscale role assign")
            .description("You cannot assign the @everyone role.");

        ctx.send(CreateReply::default().embed(embed).ephemeral(true))
            .await?;
        return Ok(());
    }

    let mut existing_tags: Vec<String> = Vec::new();

    if let Ok(db) = ctx.data().db.lock() {
        existing_tags = db
            .prepare("SELECT id FROM tailscale_tags")
            .and_then(|mut stmt| {
                stmt.query_map([], |row| row.get(0))
                    .map(|rows| rows.filter_map(|r| r.ok()).collect())
            })
            .unwrap_or_default();
    }

    if !existing_tags.contains(&tag) {
        let embed = embed::get_embed_template(embed::EmbedStatus::Error)
            .title("🛜  Tailscale role assign")
            .description(format!("Tag `{}` does not exist.", tag));

        ctx.send(CreateReply::default().embed(embed).ephemeral(true))
            .await?;
        return Ok(());
    }

    if let Ok(db) = ctx.data().db.lock() {
        db.execute(
            "DELETE FROM discord_guild_roles WHERE tailscale_tag_id = ?1",
            [tag.clone()],
        )
        .ok();

        db.execute(
            "INSERT OR IGNORE INTO discord_guilds (id) VALUES (?1)",
            (role.guild_id.get(),),
        )
        .ok();

        db.execute(
            "INSERT OR REPLACE INTO discord_guild_roles (id, discord_guild_id, tailscale_tag_id) VALUES (?1, ?2, ?3)",
            (role.id.get(), role.guild_id.get(), tag.clone()),
        ).ok();
    }

    let embed = embed::get_embed_template(embed::EmbedStatus::Success)
        .title("🛜  Tailscale role assign")
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
    if role.id.get() == role.guild_id.get() {
        let embed = embed::get_embed_template(embed::EmbedStatus::Error)
            .title("🛜  Tailscale role unassign")
            .description("You cannot unassign the @everyone role.");

        ctx.send(CreateReply::default().embed(embed).ephemeral(true))
            .await?;

        return Ok(());
    }

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

    if !deleted_role {
        let embed = embed::get_embed_template(embed::EmbedStatus::Error)
            .title("🛜  Tailscale role unassign")
            .description(format!("Role {} was not assigned.", role.mention()));

        ctx.send(CreateReply::default().embed(embed).ephemeral(true))
            .await?;

        return Ok(());
    }

    let embed = embed::get_embed_template(embed::EmbedStatus::Success)
        .title("🛜  Tailscale role unassign")
        .description(format!("Role {} has been unassigned.", role.mention()));

    ctx.send(CreateReply::default().embed(embed).ephemeral(true))
        .await?;

    Ok(())
}

/// List all role assignments
#[poise::command(slash_command)]
async fn list(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let mut embed =
        embed::get_embed_template(embed::EmbedStatus::Success).title("🛜  Tailscale role list");

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
