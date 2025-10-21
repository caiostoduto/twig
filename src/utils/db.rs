use rusqlite::{Connection, Result};
use std::fs;

pub fn connect() -> Result<Connection> {
    let conn = Connection::open("twig.sqlite")?;
    Ok(conn)
}

pub fn initialize_db(conn: &Connection) -> Result<()> {
    let schema = fs::read_to_string("resources/schema.sql").expect("Failed to read schema file");

    conn.execute_batch(&schema)?;
    Ok(())
}
