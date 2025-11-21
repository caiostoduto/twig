mod assign;
mod unassign;
mod uptime;

use crate::{Context, Data, Error};

use assign::*;
use unassign::*;
use uptime::*;

/// Minecraft command group
#[poise::command(
    slash_command,
    category = "Minecraft",
    subcommands("uptime", "assign", "unassign"),
    subcommand_required = true
)]
pub async fn minecraft(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Returns all bot commands related to Minecraft category
pub fn commands() -> Vec<poise::Command<Data, Error>> {
    vec![minecraft()]
}
