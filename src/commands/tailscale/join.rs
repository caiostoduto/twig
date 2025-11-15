use snowflaked::Generator;

use crate::{Context, Error};

/// Join a Tailscale network
#[poise::command(slash_command)]
pub async fn join(ctx: Context<'_>) -> Result<(), Error> {
    return Ok({});

    // Ask if user wants to recreate their join link if one already exists

    // Firstly, check for permissions
    // any(user.roles for user in guilds where guilds == db.guilds)

    // Secondly, get the user ID from the database or create a new one
    // let _user_id: Option<u64> = if let Ok(db) = ctx.data().db.lock() {
    //     match db
    //         .query_row(
    //             "SELECT users.id FROM users
    //             JOIN discord_users ON users.id = discord_users.user_id
    //             WHERE discord_users.id = ?1
    //             LIMIT 1",
    //             [ctx.author().id.get()],
    //             |row| row.get(0),
    //         )
    //         .ok()
    //     {
    //         Some(id) => Some(id),
    //         None => {
    //             let new_user_id = Generator::new(7332).generate();

    //             db.execute("INSERT INTO users (id) VALUES (?1)", [new_user_id])
    //                 .ok();

    //             db.execute(
    //                 "INSERT INTO discord_users (id, user_id) VALUES (?1, ?2)",
    //                 [ctx.author().id.get(), new_user_id],
    //             )
    //             .ok();

    //             Some(new_user_id)
    //         }
    //     }
    // } else {
    //     None
    // };

    // // Delete any existing join links for this user
    // // Delete the devices associated with the user

    // // Create a new join link

    // Ok(())
}
