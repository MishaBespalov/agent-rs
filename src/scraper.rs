use prometheus_parser::{MetricGroup, parse_text};
use reqwest::{Error, get};
use serde::Deserialize;
use thiserror::Error;

#[derive(Error, Debug)]
enum FetchError {
    #[error("Failed to fetch metrics: {0}")]
    Http(reqwest::Error),
    #[error("Failed to parse metrics: {0}")]
    Parse(prometheus_parser::ParserError),
}

pub async fn fetch_metrics(url: &str) -> Result<Vec<MetricGroup>, FetchError> {
    let response = reqwest::get(url).await?;
    let body = response.text().await?;
    let parsed = parse_text(&body)?;
    Ok(parsed)
}
