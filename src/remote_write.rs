use crate::metrics_agent::AgentError;
use indexmap::IndexMap;
use prometheus_parser::{GroupKey, GroupKind, MetricGroup, SimpleMetric};
use reqwest::Client;
use std::collections::BTreeMap;

pub struct RemoteWriter {
    vm_url: String,
    client: Client,
}

impl RemoteWriter {
    pub fn new(vm_url: String, client: Client) -> Self {
        RemoteWriter { vm_url, client }
    }
    pub async fn send(&self, text: String) -> Result<(), AgentError> {
        let res = self
            .client
            .post(self.vm_url.clone())
            .body(text)
            .header("Content-Type", "text/plain")
            .send()
            .await?;
        res.error_for_status()?;
        Ok(())
    }

    pub fn format_metrics_to_text(&self, metrics: &Vec<MetricGroup>) -> String {
        metrics
            .iter()
            .map(|group| format_simple_group(group))
            .collect::<Vec<_>>()
            .join("")
    }
}

pub fn format_simple_group(group: &MetricGroup) -> String {
    match &group.metrics {
        GroupKind::Gauge(metrics) => format_simple_metric(&group.name, &metrics),

        GroupKind::Counter(metrics) => format_simple_metric(&group.name, &metrics),
        _ => "skip for now".to_string(),
    }
}

pub fn format_simple_metric(
    group_name: &str,
    metrics: &IndexMap<GroupKey, SimpleMetric>,
) -> String {
    let mut result = String::new();
    for (key, metric) in metrics {
        result.push_str(&format!(
            "{}{{{}}} {}\n",
            group_name,
            format_labels(&key.labels),
            metric.value
        ))
    }
    result
}

pub fn format_labels(labels: &BTreeMap<String, String>) -> String {
    if labels.is_empty() {
        String::new()
    } else {
        labels
            .iter()
            .map(|(k, v)| format!("{}=\"{}\"", k, v))
            .collect::<Vec<_>>()
            .join(",")
    }
}
