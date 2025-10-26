mod uptime;

use crate::{Context, Data, Error, utils::config};

use uptime::*;

/// Minecraft command group
#[poise::command(
    slash_command,
    // subcommands("list", "status"),
    subcommands("uptime"),
    subcommand_required = true,
    category = "Minecraft"
)]
pub async fn minecraft(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Returns all bot commands related to Minecraft category
pub fn commands() -> Vec<poise::Command<Data, Error>> {
    // Only return Minecraft commands if all required config values are set
    if config::get_config().docker_socket.is_some() {
        vec![minecraft()]
    } else {
        vec![]
    }
}
