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
        .footer(CreateEmbedFooter::new(format!(
            "{}  •  {} @ {}",
            status_emoji,
            if config::is_debug() {
                "🛠️"
            } else {
                &config::get_config().commit_hash
            },
            &config::get_config().commit_branch
        )))
        .timestamp(Timestamp::now())
}
