use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{debug, warn};

use crate::utils::config;

/// Token refresh buffer time in seconds (5 minutes)
const TOKEN_REFRESH_BUFFER_SECS: u64 = 300;

#[derive(Debug, Clone, Deserialize)]
struct AccessToken {
    access_token: String,
    expires_in: u64, // in seconds
}

/// Errors that can occur when interacting with the Tailscale API
#[derive(Debug)]
pub enum TailscaleError {
    /// HTTP request failed
    Request(reqwest::Error),
    /// API returned an error status code with response body
    Api(u16, String),
    /// Failed to parse JSON response
    Json(serde_json::Error),
}

impl std::fmt::Display for TailscaleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TailscaleError::Request(err) => write!(f, "HTTP request error: {}", err),
            TailscaleError::Api(code, body) => {
                write!(f, "Tailscale API error (status {}): {}", code, body)
            }
            TailscaleError::Json(err) => write!(f, "JSON parsing error: {}", err),
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

/// Client for interacting with the Tailscale API
pub struct TailscaleClient {
    http: Client,
    // (token, obtained_at)
    access_token: Mutex<Option<(AccessToken, Instant)>>,
}

impl TailscaleClient {
    /// Creates a new Tailscale API client
    pub fn new() -> Self {
        Self::default()
    }

    /// Renews the OAuth access token if it's expired or about to expire
    async fn renew(&self) -> Result<(), TailscaleError> {
        // First, check if existing token is still valid without holding the lock across awaits
        let needs_refresh = {
            let token_guard = self.access_token.lock().await;
            if let Some((token, obtained_at)) = token_guard.as_ref() {
                let age = obtained_at.elapsed();
                let ttl = Duration::from_secs(token.expires_in);
                age >= ttl.saturating_sub(Duration::from_secs(TOKEN_REFRESH_BUFFER_SECS))
            } else {
                true
            }
        };

        if !needs_refresh {
            return Ok(());
        }

        let client_id = config::get_config()
            .tailscale_client_id
            .as_ref()
            .expect("TAILSCALE_CLIENT_ID environment variable must be set");
        let client_secret = config::get_config()
            .tailscale_client_secret
            .as_ref()
            .expect("TAILSCALE_CLIENT_SECRET environment variable must be set");

        let response = self
            .http
            .post(format!(
                "{}/oauth/token",
                config::get_config().tailscale_api_base
            ))
            .form(&[("client_id", client_id), ("client_secret", client_secret)])
            .send()
            .await;

        let response = match response {
            Ok(resp) => resp,
            Err(err) => {
                warn!(
                    "Failed to send Tailscale OAuth token renewal request: {}",
                    err
                );
                return Err(TailscaleError::Request(err));
            }
        };

        if !response.status().is_success() {
            let code = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            warn!(
                "Failed to renew Tailscale OAuth token: status {}, body: {}",
                code, body
            );
            return Err(TailscaleError::Api(code, body));
        }

        let token: AccessToken = match response.json().await {
            Ok(token) => token,
            Err(err) => {
                warn!("Failed to parse Tailscale OAuth token response: {}", err);
                return Err(TailscaleError::Request(err));
            }
        };
        debug!(
            "Tailscale OAuth token renewed successfully. Access token: {}",
            token.access_token
        );
        // Update the token under the lock
        let mut token_guard = self.access_token.lock().await;
        *token_guard = Some((token, Instant::now()));

        Ok(())
    }

    /// Fetches JSON data from a Tailscale API endpoint
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
            .await;

        let response = match response {
            Ok(resp) => resp,
            Err(err) => {
                warn!(
                    "Failed to send Tailscale API request to endpoint '{}': {}",
                    endpoint, err
                );
                return Err(TailscaleError::Request(err));
            }
        };

        if !response.status().is_success() {
            let code = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            warn!(
                "Tailscale API request failed for endpoint '{}': status {}, body: {}",
                endpoint, code, body
            );
            return Err(TailscaleError::Api(code, body));
        }

        let text = match response.text().await {
            Ok(text) => text,
            Err(err) => {
                warn!(
                    "Failed to read response text from Tailscale API endpoint '{}': {}",
                    endpoint, err
                );
                return Err(TailscaleError::Request(err));
            }
        };

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
        let json: Value = match serde_json::from_str(&text) {
            Ok(json) => json,
            Err(err) => {
                warn!(
                    "Failed to parse JSON response from Tailscale API endpoint '{}': {}",
                    endpoint, err
                );
                return Err(TailscaleError::Json(err));
            }
        };
        debug!("[get_json] Tailscale API response JSON: {}", json);

        Ok(json)
    }

    /// Fetches the Tailscale ACL policy file
    pub async fn get_policy_file(&self) -> Result<Value, TailscaleError> {
        self.get_json("acl").await
    }
}

impl Default for TailscaleClient {
    fn default() -> Self {
        Self {
            http: Client::new(),
            access_token: Mutex::new(None),
        }
    }
}
