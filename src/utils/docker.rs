use std::path::Path;

use reqwest::Client;

use crate::utils::config;

/// Docker client for interacting with the Docker daemon via Unix socket
pub struct DockerClient {
    client: Client,
}

impl DockerClient {
    /// Creates a new Docker client connected to the configured Unix socket
    pub fn new() -> Self {
        Self::default()
    }

    /// Pings the Docker daemon to check if it's running
    pub async fn ping(&self) -> reqwest::Result<reqwest::Response> {
        self.client.get("http://localhost/_ping").send().await
    }
}

impl Default for DockerClient {
    fn default() -> Self {
        DockerClient {
            client: Client::builder()
                .unix_socket(Path::new(
                    config::get_config()
                        .docker_socket
                        .as_ref()
                        .expect("DOCKER_SOCKET environment variable must be set"),
                ))
                .build()
                .expect("Failed to build Docker HTTP client"),
        }
    }
}
