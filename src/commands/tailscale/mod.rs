mod assign;
mod join;
mod unassign;

use crate::{Context, Data, Error, utils::config};

use assign::*;
use join::*;
use unassign::*;

use tracing::info;

/// Tailscale command group
#[poise::command(
    slash_command,
    category = "Tailscale",
    subcommands("assign", "unassign", "join"),
    subcommand_required = true
)]
pub async fn tailscale(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

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
        info!("Tailscale config detected, enabling Tailscale commands");
        vec![tailscale()]
    } else {
        info!("Tailscale config not detected, skipping Tailscale commands");
        vec![]
    }
}
