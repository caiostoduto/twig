use reqwest::{Client, header};
use tracing::debug;

use crate::utils::config;

/// Client for querying InfluxDB
pub struct InfluxDB {
    client: Client,
}

impl InfluxDB {
    /// Creates a new InfluxDB client with authentication configured
    ///
    /// # Errors
    /// Returns an error if the HTTP client cannot be built
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "Authorization",
            header::HeaderValue::from_str(&format!(
                "Bearer {}",
                config::get_config()
                    .influxdb_token
                    .as_deref()
                    .expect("Environment variable `INFLUXDB_TOKEN` not set"),
            ))
            .unwrap(),
        );

        let client = Client::builder().default_headers(headers).build()?;

        Ok(InfluxDB { client })
    }

    /// Executes a Flux query against InfluxDB
    ///
    /// # Arguments
    /// * `query` - The Flux query to execute
    ///
    /// # Returns
    /// A vector of rows, where each row is a vector of string values
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails or the response cannot be parsed
    pub async fn query(
        &self,
        query: String,
    ) -> Result<Vec<Vec<String>>, Box<dyn std::error::Error>> {
        let res = self
            .client
            .post(&format!(
                "{}/api/v2/query?org={}",
                config::get_config()
                    .influxdb_url
                    .as_deref()
                    .expect("Environment variable `INFLUXDB_URL` not set"),
                config::get_config()
                    .influxdb_org
                    .as_deref()
                    .expect("Environment variable `INFLUXDB_ORG` not set"),
            ))
            .header("Content-Type", "application/vnd.flux")
            .header("Accept", "application/csv")
            .body(query.to_string())
            .send()
            .await?;

        let text = res.text().await?;
        debug!("InfluxDB response:\n`{}`", text);

        let mut lines: Vec<Vec<String>> = text
            .lines()
            .map(|line| line.split(',').map(|s| s.to_string()).collect())
            .collect();

        lines.remove(0);
        lines.pop();

        Ok(lines)
    }
}
