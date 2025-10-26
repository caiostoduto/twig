mod ping;
mod status;

use crate::{Data, Error};

use ping::*;
use status::*;

pub fn commands() -> Vec<poise::Command<Data, Error>> {
    vec![ping(), status()]
}
