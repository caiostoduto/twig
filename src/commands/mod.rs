pub mod misc;
pub mod tailscale;

use crate::{Data, Error};

/// Returns all bot commands
pub fn commands() -> Vec<poise::Command<Data, Error>> {
    vec![misc::ping::ping(), tailscale::tailscale::tailscale()]
}
