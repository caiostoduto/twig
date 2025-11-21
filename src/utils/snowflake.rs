use snowflaked::Generator;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

// Discord epoch (January 1, 2015)
// const DISCORD_EPOCH: u64 = 1420070400000;

// fn snowflake_to_timestamp(snowflake: i64) -> u64 {
//     let snowflake_u64 = snowflake as u64;
//     (snowflake_u64 >> 22) + DISCORD_EPOCH
// }

/// Global singleton snowflake ID generator
static SNOWFLAKE_GENERATOR: OnceLock<Arc<Mutex<Generator>>> = OnceLock::new();

/// Get or initialize the global snowflake generator
pub fn get_generator() -> Arc<Mutex<Generator>> {
    SNOWFLAKE_GENERATOR
        .get_or_init(|| Arc::new(Mutex::new(Generator::new(0))))
        .clone()
}

/// Generate a new snowflake ID using the global generator
pub fn generate_id() -> i64 {
    let generator = get_generator();
    let mut g = generator.lock().unwrap();
    g.generate()
}

fn snowflake_to_timestamp(snowflake: i64) -> u64 {
    let snowflake_u64 = snowflake as u64;
    snowflake_u64 >> 22
}

pub fn is_snowflake_recent(snowflake: i64, max_age_ms: u64) -> bool {
    let timestamp_ms = snowflake_to_timestamp(snowflake);
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    now_ms.saturating_sub(timestamp_ms) <= max_age_ms
}
