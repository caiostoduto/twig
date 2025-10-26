mod ping;
mod status;

use crate::{Data, Error};

use ping::*;
use status::*;

/// Returns all bot commands related to Utilitary category
pub fn commands() -> Vec<poise::Command<Data, Error>> {
    vec![ping(), status()]
}
