use snowflaked::Generator;

use crate::{Context, Error};

/// Join a Tailscale network
#[poise::command(slash_command)]
pub async fn join(ctx: Context<'_>) -> Result<(), Error> {
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
