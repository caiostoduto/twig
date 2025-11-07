use sqlx::SqlitePool;
use tracing::info;

use crate::utils::config;

/// Opens a connection to the SQLite database
pub async fn connect() -> Result<sqlx::Pool<sqlx::Sqlite>, sqlx::Error> {
    let pool = SqlitePool::connect(&config::get_config().database_url).await;
    info!("[connect] Connected to the database successfully.");

    pool
}
