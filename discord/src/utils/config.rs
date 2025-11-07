use std::env;
use std::sync::OnceLock;

use poise::serenity_prelude::UserId;
use tracing::{debug, info};

/// Application configuration loaded from environment variables
#[derive(Debug)]
pub struct Config {
    // Discord
    pub discord_token: String,
    pub discord_owners_ids: Vec<UserId>,

    // SQLite Database URL
    pub database_url: String,

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

    // InfluxDB
    pub influxdb_url: Option<String>,
    pub influxdb_org: Option<String>,
    pub influxdb_bucket: Option<String>,
    pub influxdb_token: Option<String>,

    // gRPC
    pub grpc_port: u16,
}

/// Returns whether the application is running in debug mode
pub fn is_debug() -> bool {
    cfg!(debug_assertions)
}

impl Config {
    /// Loads configuration from environment variables
    fn from_env() -> Self {
        info!("[from_env] Loading configuration from environment variables");

        let config = Self {
            // Discord
            discord_token: env::var("DISCORD_TOKEN")
                .expect("Environment variable `DISCORD_TOKEN` not set"),
            discord_owners_ids: env::var("DISCORD_OWNER_ID")
                .unwrap_or_default()
                .split(',')
                .filter(|id| !id.trim().is_empty())
                .map(|id| {
                    id.parse()
                        .expect("Each `DISCORD_OWNER_ID` must be a valid u64 user ID")
                })
                .collect(),

            // SQLite Database URL
            database_url: env::var("DATABASE_URL").unwrap_or("sqlite:twig.sqlite".into()),

            // Tailscale
            tailscale_api_base: "https://api.tailscale.com/api/v2",
            tailscale_client_id: env::var("TAILSCALE_CLIENT_ID").ok(),
            tailscale_client_secret: env::var("TAILSCALE_CLIENT_SECRET").ok(),
            tailscale_tag: env::var("TAILSCALE_TAG").ok(),

            // Git info
            commit_hash: env!("VERGEN_GIT_SHA"),
            commit_branch: env!("VERGEN_GIT_BRANCH"),

            // Docker
            // Strip the "unix://" prefix from DOCKER_SOCKET if present, as socket paths are typically just the filesystem path
            docker_socket: env::var("DOCKER_SOCKET")
                .ok()
                .map(|val| val.strip_prefix("unix://").unwrap_or(&val).to_string()),

            // Runtime info
            start_time: std::time::Instant::now(),

            // InfluxDB
            influxdb_url: env::var("INFLUXDB_URL").ok(),
            influxdb_org: env::var("INFLUXDB_ORG").ok(),
            influxdb_bucket: env::var("INFLUXDB_BUCKET").ok(),
            influxdb_token: env::var("INFLUXDB_TOKEN").ok(),

            // gRPC
            grpc_port: env::var("GRPC_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(50051),
        };

        debug!("[from_env] Loaded configuration: {:?}", config);

        config
    }
}

// A global, thread-safe, one-time initialized config
pub static CONFIG: OnceLock<Config> = OnceLock::new();

/// Returns a reference to the global configuration instance
///
/// This function initializes the configuration on first call and returns
/// a cached reference on subsequent calls.
pub fn get_config() -> &'static Config {
    CONFIG.get_or_init(Config::from_env)
}
