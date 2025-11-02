use crate::utils::config;

use poise::serenity_prelude::{Color, CreateEmbed, CreateEmbedFooter, Timestamp};

/// Embed status type for visual indicators
pub enum EmbedStatus {
    Success,
    Error,
}

/// Creates a standardized embed template with branding and status indicator
///
/// # Arguments
/// * `status` - The status type which determines the emoji indicator
///
/// # Returns
/// A `CreateEmbed` with the bot's color scheme, footer, and timestamp
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
