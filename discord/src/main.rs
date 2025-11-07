use poise::serenity_prelude::{self as serenity};
use sqlx::SqlitePool;
use std::sync::Arc;
use tracing::{debug, error, info};

mod commands;
mod events;
mod utils;

// Types used by all command functions
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Custom user data passed to all command functions
#[derive(Clone)]
pub struct Data {
    /// Shared database connection pool
    pub db: SqlitePool,
    /// Tailscale API client
    pub ts_client: Arc<utils::tailscale::TailscaleClient>,
}

/// Custom error handler for the bot framework
async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    // This is our custom error handler
    // They are many errors that can occur, so we only handle the ones we want to customize
    // and forward the rest to the default handler
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx, .. } => {
            error!("Error in command `{}`: {:?}", ctx.command().name, error);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                error!("Error while handling error: {}", e)
            }
        }
    }
}

/// Starts and runs the Discord bot
pub async fn start() {
    info!("[start] Starting Twig bot");

    // FrameworkOptions contains all of poise's configuration option in one struct
    // Every option can be omitted to use its default value
    let options = poise::FrameworkOptions {
        commands: commands::commands(),
        // Set bot owners who have special permissions
        owners: {
            let owners = std::collections::HashSet::from_iter(
                utils::config::get_config().discord_owners_ids.clone(),
            );

            info!("[start] Bot owners count: {}", owners.len());
            debug!("[start] Bot owners IDs: {:?}", owners);

            owners
        },
        // The global error handler for all error cases that may occur
        on_error: |error| Box::pin(on_error(error)),
        // This code is run before every command
        pre_command: |ctx| {
            Box::pin(async move {
                info!(
                    "[pre_command::{}] {} ({}) @ {}",
                    ctx.command().qualified_name,
                    ctx.author().name,
                    ctx.author().id,
                    ctx.guild()
                        .map(|g| g.name.to_string())
                        .unwrap_or_else(|| "DM".to_string())
                );
            })
        },
        // This code is run after a command if it was successful (returned Ok)
        post_command: |ctx| {
            Box::pin(async move {
                info!(
                    "[post_command::{}] {} ({}) @ {}",
                    ctx.command().qualified_name,
                    ctx.author().name,
                    ctx.author().id,
                    ctx.guild()
                        .map(|g| g.name.to_string())
                        .unwrap_or_else(|| "DM".to_string())
                );
            })
        },
        // Enforce command checks even for owners (enforced by default)
        // Set to true to bypass checks, which is useful for testing
        skip_checks_for_owners: false,
        event_handler: |ctx, event, framework, data| {
            Box::pin(events::event_handler(ctx, event, framework, data))
        },
        ..Default::default()
    };

    let framework = poise::Framework::builder()
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                info!("[framework::setup] Logged in as {}", _ready.user.name);

                // Initialize database
                let pool = utils::db::connect()
                    .await
                    .expect("Failed to connect to database");

                info!("[framework::setup] Database initialized successfully");

                info!(
                    "[framework::setup] Registering ({}) global commands",
                    framework.options().commands.len()
                );

                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                // Create the Data structure
                let data = Arc::new(Data {
                    db: pool,
                    ts_client: Arc::new(utils::tailscale::TailscaleClient::new()),
                });

                Ok(Arc::try_unwrap(data).unwrap_or_else(|arc| (*arc).clone()))
            })
        })
        .options(options)
        .build();

    let token = &utils::config::get_config().discord_token;
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::GUILD_MEMBERS;

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    info!("[start] Starting autosharded client");
    client.unwrap().start_autosharded().await.unwrap();
}

#[dotenvy::load(required = false)]
#[tokio::main]
async fn main() {
    // Initialize logging with environment variable support
    // Set RUST_LOG environment variable to control log levels
    // Examples:
    //   RUST_LOG=debug       - Show all debug and higher logs
    //   RUST_LOG=twig=trace  - Show trace logs only for twig crate
    //   RUST_LOG=info        - Show info and higher (default)
    use tracing_subscriber::{EnvFilter, fmt};

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    fmt()
        .with_env_filter(env_filter)
        .with_target(true)
        .with_thread_ids(false)
        .with_line_number(false)
        .with_file(false)
        .init();

    start().await;
}
