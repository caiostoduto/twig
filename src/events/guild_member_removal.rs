use crate::{Data, Error};
use poise::serenity_prelude as serenity;

/// Handles when a member leaves or is removed from a guild
pub async fn handle(
    _ctx: &serenity::Context,
    guild_id: &serenity::GuildId,
    user: &serenity::User,
    _data: &Data,
) -> Result<(), Error> {
    println!("User {} ({}) left guild {}", user.name, user.id, guild_id);

    Ok(())
}
