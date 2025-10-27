mod uptime;

use crate::{Context, Data, Error};

use uptime::*;

/// Minecraft command group
#[poise::command(
    slash_command,
    subcommands("uptime"),
    subcommand_required = true,
    category = "Minecraft"
)]
pub async fn minecraft(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Returns all bot commands related to Minecraft category
pub fn commands() -> Vec<poise::Command<Data, Error>> {
    vec![minecraft()]
}
