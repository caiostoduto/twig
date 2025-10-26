use std::path::Path;

use reqwest::Client;

use crate::utils::config;

pub struct DockerClient {
    client: Client,
}

#[derive(serde::Deserialize)]
pub struct DockerContainer {
    #[serde(rename = "Id")]
    pub id: String,
    #[serde(rename = "Labels")]
    pub labels: std::collections::HashMap<String, String>,
}

#[derive(serde::Deserialize)]
pub struct DockerContainerStateHealth {
    #[serde(rename = "Status")]
    pub status: String,
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DockerContainerStateStatus {
    Created,
    Running,
    Paused,
    Restarting,
    Removing,
    Exited,
    Dead,
}

#[derive(serde::Deserialize)]
pub struct DockerContainerState {
    #[serde(rename = "Health")]
    pub health: Option<DockerContainerStateHealth>,
    #[serde(rename = "Status")]
    pub status: DockerContainerStateStatus,
    #[serde(rename = "StartedAt")]
    pub started_at: String,
    #[serde(rename = "FinishedAt")]
    pub finished_at: String,
}

#[derive(serde::Deserialize)]
pub struct DockerContainerDetails {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "State")]
    pub state: DockerContainerState,
}

impl DockerClient {
    pub fn new() -> Self {
        DockerClient {
            client: Client::builder()
                .unix_socket(Path::new(
                    config::get_config().docker_socket.as_ref().unwrap(),
                ))
                .build()
                .unwrap(),
        }
    }

    pub async fn ping(&self) -> reqwest::Result<reqwest::Response> {
        self.client.get("http://localhost/_ping").send().await
    }

    pub async fn list_containers(&self) -> Result<Vec<DockerContainer>, reqwest::Error> {
        let response = self
            .client
            .get("http://localhost/containers/json")
            .send()
            .await?;

        let containers = response.json::<Vec<DockerContainer>>().await?;
        Ok(containers)
    }

    pub async fn inspect_container(
        &self,
        container_id: &str,
    ) -> Result<DockerContainerDetails, reqwest::Error> {
        let url = format!("http://localhost/containers/{}/json", container_id);
        let response = self.client.get(&url).send().await?;
        let container_info = response.json::<DockerContainerDetails>().await?;
        Ok(container_info)
    }
}
