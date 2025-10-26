pub mod tailscale;
pub mod utilitary;

use crate::{Data, Error};

/// Returns all bot commands
pub fn commands() -> Vec<poise::Command<Data, Error>> {
    vec![
        utilitary::ping(),
        utilitary::status(),
        tailscale::tailscale(),
    ]
}
