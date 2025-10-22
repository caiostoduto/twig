use std::env;
use std::sync::OnceLock;

use poise::serenity_prelude::UserId;

#[derive(Debug)]
pub struct Config {
    pub discord_token: String,
    pub discord_owners_ids: Vec<UserId>,

    pub tailscale_api_base: &'static str,
    pub tailscale_client_id: String,
    pub tailscale_client_secret: String,
    pub tailscale_tag: String,

    pub commit_hash: String,
    pub commit_branch: String,
}

pub fn is_debug() -> bool {
    cfg!(debug_assertions)
}

impl Config {
    fn from_env() -> Self {
        Self {
            discord_token: env::var("DISCORD_TOKEN")
                .expect("Environment variable `DISCORD_TOKEN` not set"),
            discord_owners_ids: env::var("DISCORD_OWNER_ID")
                .expect("Environment variable `DISCORD_OWNER_ID` not set")
                .split(',')
                .map(|id| {
                    id.parse()
                        .expect("Each `DISCORD_OWNER_ID` must be a valid u64 user ID")
                })
                .collect(),

            tailscale_api_base: "https://api.tailscale.com/api/v2",
            tailscale_client_id: env::var("TAILSCALE_CLIENT_ID")
                .expect("Environment variable `TAILSCALE_CLIENT_ID` not set"),
            tailscale_client_secret: env::var("TAILSCALE_CLIENT_SECRET")
                .expect("Environment variable `TAILSCALE_CLIENT_SECRET` not set"),
            tailscale_tag: env::var("TAILSCALE_TAG")
                .expect("Environment variable `TAILSCALE_TAG` not set"),

            commit_hash: env!("VERGEN_GIT_SHA").to_string(),
            commit_branch: env!("VERGEN_GIT_BRANCH").to_string(),
        }
    }
}

// A global, thread-safe, one-time initialized config
pub static CONFIG: OnceLock<Config> = OnceLock::new();

pub fn get_config() -> &'static Config {
    CONFIG.get_or_init(Config::from_env)
}
