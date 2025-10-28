mod minecraft;
mod tailscale;
mod utilitary;

use crate::{Data, Error};

/// Returns all bot commands
pub fn commands() -> Vec<poise::Command<Data, Error>> {
    let mut commands = Vec::new();

    commands.extend(minecraft::commands());
    commands.extend(tailscale::commands());
    commands.extend(utilitary::commands());

    commands
}
