use tracing::debug;

use crate::{Context, Error};

/// Check if the user is the bot owner
pub async fn is_owner(ctx: Context<'_>) -> Result<bool, Error> {
    let owner = ctx.framework().options().owners.contains(&ctx.author().id);

    if !owner {
        debug!("[is_owner] Attempted to use owner-only command, but user is not owner");
        ctx.say("❌ This command can only be used by the bot owner.")
            .await?;
    }

    Ok(owner)
}
