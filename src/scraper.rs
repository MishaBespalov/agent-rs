use crate::metrics_agent::AgentError;
use std::time::Duration;
use tokio_retry::{Retry, strategy::ExponentialBackoff};

use prometheus_parser::{MetricGroup, parse_text};
use reqwest::Client;

pub struct TargetScraper {
    url: String,
    client: Client,
    timeout: Duration,
    max_retries: usize,
}

impl TargetScraper {
    pub fn new(url: String, client: Client, timeout: Duration, max_retries: usize) -> Self {
        TargetScraper {
            url,
            client,
            timeout,
            max_retries,
        }
    }
    pub async fn scrape(&self) -> Result<Vec<MetricGroup>, AgentError> {
        let strategy = ExponentialBackoff::from_millis(100)
            .max_delay(Duration::from_secs(10))
            .take(self.max_retries);

        Retry::spawn(strategy, || async {
            tokio::time::timeout(self.timeout, fetch_metrics(self.client.clone(), &self.url))
                .await?
        })
        .await
    }
}

pub async fn fetch_metrics(client: Client, url: &str) -> Result<Vec<MetricGroup>, AgentError> {
    let response = client.get(url).send().await?;
    let body = response.text().await?;
    let parsed = parse_text(&body)?;
    Ok(parsed)
}
