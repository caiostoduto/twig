pub mod tailscale;
pub mod utilitary;

use crate::{Data, Error};

/// Returns all bot commands
pub fn commands() -> Vec<poise::Command<Data, Error>> {
    let mut commands = Vec::new();

    commands.extend(utilitary::commands());
    commands.extend(tailscale::commands());

    commands
}
