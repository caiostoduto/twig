use std::path::Path;

use reqwest::Client;

use crate::utils::config;

pub struct DockerClient {
    client: Client,
}

impl DockerClient {
    pub fn new() -> Self {
        DockerClient {
            client: Client::builder()
                .unix_socket(Path::new(
                    config::get_config()
                        .docker_socket
                        .as_ref()
                        .expect("DOCKER_SOCKET environment variable must be set"),
                ))
                .build()
                .unwrap(),
        }
    }

    pub async fn ping(&self) -> reqwest::Result<reqwest::Response> {
        self.client.get("http://localhost/_ping").send().await
    }
}
