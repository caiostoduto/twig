use axum::extract::{Query, State};
use axum::http::StatusCode;
use reqwest::Url;
use serde::Deserialize;
use tracing::warn;

use crate::grpc::stream::authenticated;
use crate::utils::config;
use crate::utils::snowflake::is_snowflake_recent;

#[derive(Deserialize)]
pub struct OAuthParams {
    pub code: Option<String>,
    pub state: Option<String>,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct DiscordUser {
    id: String,
}

pub async fn oauth_callback(
    State(app_state): State<super::AppState>,
    Query(params): Query<OAuthParams>,
) -> Result<String, (StatusCode, String)> {
    // Access Discord context
    let code = match params.code {
        Some(c) if !c.is_empty() => c,
        _ => return Err((StatusCode::BAD_REQUEST, "No code provided".to_string())),
    };
    let state = match params.state {
        Some(s) if !s.is_empty() => s,
        _ => return Err((StatusCode::BAD_REQUEST, "No state provided".to_string())),
    };

    // Check if is a valid state in db
    let Some((minecraft_user_id, minecraft_registrations_id)) = sqlx::query!(
        "SELECT minecraft_users.id as user_id, minecraft_registrations.id as regs_id FROM minecraft_registrations JOIN minecraft_users ON minecraft_registrations.minecraft_user_id = minecraft_users.id WHERE state_token = $1 AND minecraft_users.discord_user_id IS NULL",
        state
    ).fetch_one(&app_state.data.db).await.map_err(|_| {
        (StatusCode::BAD_REQUEST, "Invalid state token".to_string())
    }).map(|record| (record.user_id, record.regs_id)).ok() else {
        return Err((StatusCode::BAD_REQUEST, "Invalid state token".to_string()));
    };

    // Check if registration is less than 5 minutes old
    const FIVE_MINUTES_MS: u64 = 5 * 60 * 1000;
    if !is_snowflake_recent(minecraft_registrations_id, FIVE_MINUTES_MS) {
        return Err((
            StatusCode::BAD_REQUEST,
            "Registration token expired".to_string(),
        ));
    }

    if config::get_config().discord_oauth_client_id.is_none()
        || config::get_config().discord_oauth_client_secret.is_none()
    {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Discord OAuth is not configured".to_string(),
        ));
    }

    let callback_url = Url::parse(config::get_config().app_url.as_deref().unwrap())
        .unwrap()
        .join("/discord/callback")
        .unwrap();

    let client = reqwest::Client::new();

    let token_response = client
        .post("https://discord.com/api/oauth2/token")
        .form(&[
            (
                "client_id",
                config::get_config().discord_oauth_client_id.as_deref(),
            ),
            (
                "client_secret",
                config::get_config().discord_oauth_client_secret.as_deref(),
            ),
            ("grant_type", Some("authorization_code")),
            ("code", Some(&code)),
            ("redirect_uri", Some(callback_url.as_str())),
            ("scope", Some("identify")),
        ])
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_GATEWAY,
                format!("Failed to contact Discord: {}", e),
            )
        })?
        .json::<TokenResponse>()
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_GATEWAY,
                format!("Failed to parse Discord token response: {}", e),
            )
        })?;

    let user = client
        .get("https://discord.com/api/users/@me")
        .bearer_auth(&token_response.access_token)
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_GATEWAY,
                format!("Failed to fetch Discord user: {}", e),
            )
        })?
        .json::<DiscordUser>()
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_GATEWAY,
                format!("Failed to parse Discord user response: {}", e),
            )
        })?;

    if let Some(_existing) = sqlx::query!(
        "SELECT id FROM minecraft_users WHERE 
        player_name = (SELECT player_name FROM minecraft_users WHERE id = (SELECT minecraft_user_id FROM minecraft_registrations WHERE state_token = $1)) AND discord_user_id != $2",
        state,
        user.id
    ).fetch_optional(&app_state.data.db).await.unwrap() {
        warn!("[Discord OAuth] Discord user {} is already linked to another Minecraft account.", user.id);

        return Err((StatusCode::BAD_REQUEST, "Este usuário do Discord já está vinculado a outra conta Minecraft.".to_string()));
    }

    // Insert Discord user if not exists
    let _ = sqlx::query!(
        "INSERT OR IGNORE INTO discord_users (id) VALUES ($1)",
        user.id
    )
    .execute(&app_state.data.db)
    .await
    .unwrap();

    // Link Discord user to Minecraft user
    let _ = sqlx::query!(
        "UPDATE minecraft_users SET discord_user_id = $1 WHERE id = (SELECT minecraft_user_id FROM minecraft_registrations WHERE state_token = $2)",
        user.id,
        state
    )
    .execute(&app_state.data.db)
    .await.unwrap();

    authenticated::broadcast_event(app_state.data, minecraft_user_id).await;

    Ok("Sucesso!".to_string())
}
