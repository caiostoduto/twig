use crate::utils::config;

use poise::serenity_prelude::{Color, CreateEmbed, CreateEmbedFooter, Timestamp};

pub enum EmbedStatus {
    Success,
    Error,
}

pub fn get_embed_template(status: EmbedStatus) -> CreateEmbed {
    let status_emoji = match status {
        EmbedStatus::Success => "✅",
        EmbedStatus::Error => "⚠️",
    };

    CreateEmbed::new()
        .color(Color::new(0x632434))
        .footer(CreateEmbedFooter::new(if config::is_debug() {
            format!("{}  •  develop", status_emoji)
        } else {
            format!("{}  •  {}", status_emoji, &config::get_config().commit_hash)
        }))
        .timestamp(Timestamp::now())
}
