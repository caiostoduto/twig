use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

use crate::utils::config;

#[derive(Debug, Clone, Deserialize)]
struct AccessToken {
    access_token: String,
    expires_in: u64, // in seconds
}

#[derive(Debug)]
pub enum TailscaleError {
    Request(reqwest::Error),
    Api(u16, String),
    Json(serde_json::Error),
}

impl std::fmt::Display for TailscaleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TailscaleError::Request(err) => write!(f, "HTTP request error: {}", err),
            TailscaleError::Api(code, body) => {
                write!(f, "Tailscale API error (status {}): {}", code, body)
            }
            TailscaleError::Json(err) => write!(f, "JSON error: {}", err),
        }
    }
}

impl std::error::Error for TailscaleError {}

impl From<reqwest::Error> for TailscaleError {
    fn from(value: reqwest::Error) -> Self {
        TailscaleError::Request(value)
    }
}

impl From<serde_json::Error> for TailscaleError {
    fn from(value: serde_json::Error) -> Self {
        TailscaleError::Json(value)
    }
}

pub struct TailscaleClient {
    http: Client,
    // (token, obtained_at)
    access_token: Mutex<Option<(AccessToken, Instant)>>,
}

impl TailscaleClient {
    pub fn new() -> Self {
        Self {
            http: Client::new(),
            access_token: Mutex::new(None),
        }
    }

    async fn renew(&self) -> Result<(), TailscaleError> {
        // First, check if existing token is still valid without holding the lock across awaits
        let needs_refresh = {
            let token_guard = self.access_token.lock().await;
            if let Some((token, obtained_at)) = token_guard.as_ref() {
                let age = obtained_at.elapsed();
                let ttl = Duration::from_secs(token.expires_in);
                age >= ttl.saturating_sub(Duration::from_secs(300))
            } else {
                true
            }
        };

        if !needs_refresh {
            return Ok(());
        }

        let client_id = &config::get_config().tailscale_client_id;
        let client_secret = &config::get_config().tailscale_client_secret;

        let response = self
            .http
            .post(format!(
                "{}/oauth/token",
                config::get_config().tailscale_api_base
            ))
            .form(&[
                ("client_id", client_id.as_str()),
                ("client_secret", client_secret.as_str()),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            let code = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(TailscaleError::Api(code, body));
        }

        let token: AccessToken = response.json().await?;
        // Update the token under the lock
        let mut token_guard = self.access_token.lock().await;
        *token_guard = Some((token, Instant::now()));

        Ok(())
    }

    async fn get_json(&self, endpoint: &str) -> Result<Value, TailscaleError> {
        self.renew().await?;

        // Read the access token string under lock (clone the String to avoid holding the lock)
        let token = {
            let token_guard = self.access_token.lock().await;
            token_guard
                .as_ref()
                .map(|(t, _)| t.access_token.clone())
                .expect("token must be present after renew")
        };

        let response = self
            .http
            .get(format!(
                "{}/tailnet/-/{}",
                config::get_config().tailscale_api_base,
                endpoint
            ))
            .bearer_auth(&token)
            .send()
            .await?;

        if !response.status().is_success() {
            let code = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();

            return Err(TailscaleError::Api(code, body));
        }

        let text = response.text().await?;

        let text: String = text
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n");

        // Remove trailing commas before closing brackets/braces
        let text = text
            .replace(",\n]", "\n]")
            .replace(",\n}", "\n}")
            .replace(",]", "]")
            .replace(",}", "}");

        // Try to parse as JSON
        Ok(serde_json::from_str(&text)?)
    }

    pub async fn get_policy_file(&self) -> Result<Value, TailscaleError> {
        self.get_json("acl").await
    }
}
