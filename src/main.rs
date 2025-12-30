use std::time;

mod metrics_agent;
mod remote_write;
mod scraper;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let reqwest_client = reqwest::Client::new();
    let scraper = scraper::TargetScraper::new(
        "http://127.0.0.1:9100/metrics".to_string(),
        reqwest_client.clone(),
        time::Duration::from_secs(5),
        10,
    );
    let writer = remote_write::RemoteWriter::new(
        "http://127.0.0.1:8428/api/v1/import/prometheus".to_string(),
        reqwest_client,
    );
    let metrics_agent =
        metrics_agent::MetricsAgent::new(writer, scraper, time::Duration::from_secs(30));
    _ = metrics_agent.run().await;
}
