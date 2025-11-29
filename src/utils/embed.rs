use crate::utils::config;

use poise::serenity_prelude::{Color, CreateEmbed, CreateEmbedFooter, Timestamp};

/// Creates a success embed template
/// # Returns
/// A `CreateEmbed` with success indicators
pub fn success() -> CreateEmbed {
    create_embed_template(EmbedStatus::Success)
}

/// Creates an error embed template
/// # Returns
/// A `CreateEmbed` with error indicators
pub fn warn() -> CreateEmbed {
    create_embed_template(EmbedStatus::Warn)
}

/// Embed status type for visual indicators
enum EmbedStatus {
    Success,
    Warn,
}

/// Creates a standardized embed template with branding and status indicator
///
/// # Arguments
/// * `status` - The status type which determines the emoji indicator
///
/// # Returns
/// A `CreateEmbed` with the bot's color scheme, footer, and timestamp
fn create_embed_template(status: EmbedStatus) -> CreateEmbed {
    let status_emoji = match status {
        EmbedStatus::Success => "✅",
        EmbedStatus::Warn => "⚠️",
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
