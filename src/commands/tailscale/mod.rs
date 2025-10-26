mod tailscale;

use crate::{Data, Error, utils::config};

use tailscale::*;

/// Returns all bot commands related to Tailscale category
pub fn commands() -> Vec<poise::Command<Data, Error>> {
    // Only return Tailscale commands if all required config values are set
    if [
        &config::get_config().tailscale_client_id,
        &config::get_config().tailscale_client_secret,
        &config::get_config().tailscale_tag,
    ]
    .iter()
    .all(|f| f.is_some())
    {
        vec![tailscale()]
    } else {
        vec![]
    }
}
