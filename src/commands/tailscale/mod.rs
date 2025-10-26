mod tailscale;

use crate::{Data, Error, utils::config};

use tailscale::*;

pub fn commands() -> Vec<poise::Command<Data, Error>> {
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
