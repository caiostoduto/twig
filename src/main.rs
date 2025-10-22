use poise::serenity_prelude::{self as serenity};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use tracing::{error, info};

mod commands;
mod events;
mod utils;

// Types used by all command functions
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

// Custom user data passed to all command functions
pub struct Data {
    pub db: Arc<Mutex<Connection>>,
    pub tailscale_client: Arc<utils::tailscale::TailscaleClient>,
}

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

pub async fn start() {
    // FrameworkOptions contains all of poise's configuration option in one struct
    // Every option can be omitted to use its default value
    let options = poise::FrameworkOptions {
        commands: commands::commands(),
        // Set bot owners who have special permissions
        owners: {
            std::collections::HashSet::from_iter(
                utils::config::get_config().discord_owners_ids.clone(),
            )
        },
        // The global error handler for all error cases that may occur
        on_error: |error| Box::pin(on_error(error)),
        // This code is run before every command
        pre_command: |ctx| {
            Box::pin(async move {
                info!("Executing command {}...", ctx.command().qualified_name);
            })
        },
        // This code is run after a command if it was successful (returned Ok)
        post_command: |ctx| {
            Box::pin(async move {
                info!("Executed command {}!", ctx.command().qualified_name);
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
                info!("Logged in as {}", _ready.user.name);

                // Initialize database
                let conn = utils::db::connect().expect("Failed to connect to database");
                utils::db::initialize_db(&conn).expect("Failed to initialize database");

                info!("Database initialized successfully");

                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    db: Arc::new(Mutex::new(conn)),
                    tailscale_client: Arc::new(utils::tailscale::TailscaleClient::new()),
                })
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

    client.unwrap().start_autosharded().await.unwrap();
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    start().await;
}
