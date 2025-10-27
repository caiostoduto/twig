use reqwest::{Client, header};

use crate::utils::config;

pub struct InfluxDB {
    client: Client,
}

impl InfluxDB {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "Authorization",
            header::HeaderValue::from_str(&format!(
                "Bearer {}",
                config::get_config()
                    .influxdb_token
                    .as_deref()
                    .expect("INFLUXDB_TOKEN environment variable must be set"),
            ))
            .unwrap(),
        );

        let client = Client::builder().default_headers(headers).build()?;

        Ok(InfluxDB { client })
    }

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
                    .expect("INFLUXDB_URL environment variable must be set"),
                config::get_config()
                    .influxdb_org
                    .as_deref()
                    .expect("INFLUXDB_ORG environment variable must be set"),
            ))
            .header("Content-Type", "application/vnd.flux")
            .header("Accept", "application/csv")
            .body(query.to_string())
            .send()
            .await
            .unwrap();

        let text = res.text().await?;
        // println!("Response Text:\n{}", text);

        let mut lines: Vec<Vec<String>> = text
            .lines()
            .map(|line| line.split(',').map(|s| s.to_string()).collect())
            .collect();

        lines.remove(0);
        lines.pop();

        Ok(lines)
    }
}
