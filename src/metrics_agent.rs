use crate::metrics_formatter::MetricsFormatter;
use crate::remote_write::RemoteWriter;
use crate::scraper::TargetScraper;
use anyhow::{Context, Result};
use prometheus_parser::{
    GroupKey, GroupKind, HistogramMetric, MetricGroup, SimpleMetric, SummaryMetric,
};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tracing::{debug, error};

pub struct MetricsMessage {
    pub target_url: String,
    pub metrics: Vec<MetricGroup>,
    pub scraped_at: Instant,
}

pub struct MetricsAgent {
    writer: RemoteWriter,
    scraper: TargetScraper,
    formatter: MetricsFormatter,
    pub interval: Duration,
}

impl MetricsAgent {
    pub fn new(
        writer: RemoteWriter,
        formatter: MetricsFormatter,
        scraper: TargetScraper,
        interval: Duration,
    ) -> Self {
        MetricsAgent {
            writer,
            scraper,
            formatter,
            interval,
        }
    }

    pub async fn scrape(&self, tx: mpsc::Sender<MetricsMessage>) -> Result<()> {
        let metrics = self.scraper.scrape().await?;
        let scraped_at = Instant::now();

        let metric_message = MetricsMessage {
            metrics,
            target_url: self.scraper.url.clone(),
            scraped_at,
        };
        tx.send(metric_message).await?;
        Ok(())
    }

    pub async fn format(
        &self,
        mut rx: mpsc::Receiver<MetricsMessage>,
        tx: mpsc::Sender<String>,
    ) -> Result<()> {
        let mut batch = Vec::with_capacity(30);
        while let Some(metric_message) = rx.recv().await {
            batch.push(metric_message);

            if batch.len() >= 32 {
                let formatted_text = self.formatter.format_batch(&batch);
                tx.send(formatted_text).await?;
                batch.clear();
            }
        }
        if !batch.is_empty() {
            tx.send(self.formatter.format_batch(&batch));
        }
        Ok(())
    }

    pub async fn write(&self, mut rx: mpsc::Receiver<String>) -> Result<()> {
        let mut batch = Vec::with_capacity(30);
        while let Some(formatted_text) = rx.recv().await {
            batch.push(formatted_text);
            if batch.len() >= 128 {
                self.writer.send(batch.concat()).await?;
                batch.clear();
            }
        }
        if !batch.is_empty() {
            self.writer.send(batch.concat()).await?;
            batch.clear();
        }
        Ok(())
    }
}
