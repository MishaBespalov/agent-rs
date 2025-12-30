use crate::remote_write::RemoteWriter;
use crate::scraper::TargetScraper;
use std::time::Duration;
use thiserror::Error;
use tracing::{debug, error};

pub struct MetricsAgent {
    writer: RemoteWriter,
    scraper: TargetScraper,
    interval: Duration,
}

#[derive(Error, Debug)]
pub enum AgentError {
    #[error("Http request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Failed to parse metrics: {0}")]
    Parse(#[from] prometheus_parser::ParserError),
    #[error("Timeout fetching metrics: {0}")]
    Timeout(#[from] tokio::time::error::Elapsed),
}

impl MetricsAgent {
    pub fn new(writer: RemoteWriter, scraper: TargetScraper, interval: Duration) -> Self {
        MetricsAgent {
            writer,
            scraper,
            interval,
        }
    }
    pub async fn run(&self) -> Result<(), AgentError> {
        let mut interval = tokio::time::interval(self.interval);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            let metrics = match self.scraper.scrape().await {
                Ok(metrics) => {
                    debug!("Successfully scraped {} metrics", metrics.len());
                    metrics
                }
                Err(e) => {
                    error!("Scraping failed: {}", e);
                    continue;
                }
            };
            let text = self.writer.format_metrics_to_text(&metrics);
            if let Err(e) = self.writer.send(text).await {
                error!("Remote write failed {}", e);
            }
        }
    }
}
