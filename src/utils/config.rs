use std::env;
use std::sync::OnceLock;

use poise::serenity_prelude::UserId;

#[derive(Debug)]
pub struct Config {
    // Discord
    pub discord_token: String,
    pub discord_owners_ids: Vec<UserId>,

    // Tailscale
    pub tailscale_api_base: &'static str,
    pub tailscale_client_id: Option<String>,
    pub tailscale_client_secret: Option<String>,
    pub tailscale_tag: Option<String>,

    // Git info (set at build time)
    pub commit_hash: &'static str,
    pub commit_branch: &'static str,

    // Docker
    pub docker_socket: Option<String>,

    // Runtime info
    pub start_time: std::time::Instant,
}

pub fn is_debug() -> bool {
    cfg!(debug_assertions)
}

impl Config {
    fn from_env() -> Self {
        Self {
            // Discord
            discord_token: env::var("DISCORD_TOKEN")
                .expect("Environment variable `DISCORD_TOKEN` not set"),
            discord_owners_ids: env::var("DISCORD_OWNER_ID")
                .unwrap_or("".to_string())
                .split(',')
                .map(|id| {
                    id.parse()
                        .expect("Each `DISCORD_OWNER_ID` must be a valid u64 user ID")
                })
                .collect(),

            // Tailscale
            tailscale_api_base: "https://api.tailscale.com/api/v2",
            tailscale_client_id: env::var("TAILSCALE_CLIENT_ID").ok(),
            tailscale_client_secret: env::var("TAILSCALE_CLIENT_SECRET").ok(),
            tailscale_tag: env::var("TAILSCALE_TAG").ok(),

            // Git info
            commit_hash: env!("VERGEN_GIT_SHA"),
            commit_branch: env!("VERGEN_GIT_BRANCH"),

            // Docker
            docker_socket: match env::var("DOCKER_SOCKET").ok() {
                Some(val) => Some(val.strip_prefix("unix://").unwrap_or(&val).to_string()),
                _ => None,
            },

            // Runtime info
            start_time: std::time::Instant::now(),
        }
    }
}

// A global, thread-safe, one-time initialized config
pub static CONFIG: OnceLock<Config> = OnceLock::new();

pub fn get_config() -> &'static Config {
    CONFIG.get_or_init(Config::from_env)
}
