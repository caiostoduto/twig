use reqwest::{Client, header};

use crate::utils::config;

pub struct InfluxDB {
    client: Client,
}

impl InfluxDB {
    pub fn new() -> Self {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "Authorization",
            header::HeaderValue::from_str(&format!(
                "Bearer {}",
                config::get_config().influxdb_token.as_deref().unwrap()
            ))
            .unwrap(),
        );

        let client = Client::builder().default_headers(headers).build().unwrap();

        InfluxDB { client }
    }

    pub async fn query(&self, query: String) -> Vec<Vec<String>> {
        let res = self
            .client
            .post(&format!(
                "{}/api/v2/query?org={}",
                config::get_config().influxdb_url.as_deref().unwrap(),
                config::get_config().influxdb_org.as_deref().unwrap()
            ))
            .header("Content-Type", "application/vnd.flux")
            .header("Accept", "application/csv")
            .body(query.to_string())
            .send()
            .await
            .unwrap();

        let text = res.text().await.unwrap();
        // println!("Response Text:\n{}", text);

        let mut lines: Vec<Vec<String>> = text
            .lines()
            .map(|line| line.split(",").map(|s| s.to_string()).collect())
            .collect();

        lines.remove(0);
        lines.pop();

        lines
    }
}
