use rusqlite::{Connection, Result};
use std::fs;
use tracing::info;

/// Opens a connection to the SQLite database
pub fn connect() -> Result<Connection> {
    let conn = Connection::open("twig.sqlite")?;
    info!("[connect] Connected to the database successfully.");

    Ok(conn)
}

/// Initializes the database with the schema from the migration file
///
/// # Errors
/// Returns an error if the migration file cannot be read or if the SQL execution fails
pub fn initialize_db(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    let schema = fs::read_to_string("migrations/initial_migration.sql")
        .map_err(|e| format!("Failed to read migration file: {}", e))?;

    conn.execute_batch(&schema)
        .map_err(|e| format!("Failed to execute migration: {}", e))?;

    info!("[initialize_db] Database initialized successfully with the migration schema.");

    Ok(())
}
