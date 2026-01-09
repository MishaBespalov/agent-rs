use crate::metrics_agent::MetricsMessage;
use anyhow::Result;
use std::sync::Arc;
use std::time;
use tokio::sync::mpsc;

mod metrics_agent;
mod metrics_formatter;
mod remote_write;
mod scraper;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let reqwest_client = reqwest::Client::new();
    let scraper = scraper::TargetScraper::new(
        "http://127.0.0.1:9100/metrics".to_string(),
        reqwest_client.clone(),
        time::Duration::from_secs(5),
        10,
    );
    let (scrape_tx, scrape_rx) = mpsc::channel::<MetricsMessage>(32);
    let formatter = metrics_formatter::MetricsFormatter {};
    let (format_tx, format_rx) = mpsc::channel::<String>(32);

    let writer = remote_write::RemoteWriter::new(
        "http://127.0.0.1:8428/api/v1/import/prometheus".to_string(),
        reqwest_client,
    );
    let metrics_agent = Arc::new(metrics_agent::MetricsAgent::new(
        writer,
        formatter,
        scraper,
        time::Duration::from_secs(30),
    ));
    let metric_scraper_clone = Arc::clone(&metrics_agent);
    let scraper_handle = tokio::spawn(async move {
        loop {
            metric_scraper_clone.scrape(scrape_tx.clone()).await;
            tokio::time::sleep(metric_scraper_clone.interval).await;
        }
    });

    let metric_formatter_clone = Arc::clone(&metrics_agent);
    let formater_handle =
        tokio::spawn(async move { metric_formatter_clone.format(scrape_rx, format_tx).await });

    let metric_writer_clone = Arc::clone(&metrics_agent);
    let writer_handle = tokio::spawn(async move { metric_writer_clone.write(format_rx).await });
    tokio::try_join!(scraper_handle, formater_handle, writer_handle)?;
    Ok(())
}
