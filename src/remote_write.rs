use crate::metrics_agent::AgentError;
use indexmap::IndexMap;
use prometheus_parser::{GroupKey, GroupKind, MetricGroup, SimpleMetric, SummaryMetric, HistogramMetric};
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

    pub fn format_metrics_to_text(&self, metrics: &[MetricGroup]) -> String {
        metrics
            .iter()
            .map(format_simple_group)
            .collect::<Vec<_>>()
            .join("")
    }
}

pub fn format_simple_group(group: &MetricGroup) -> String {
    match &group.metrics {
        GroupKind::Gauge(metrics) => format_simple_metric(&group.name, metrics),
        GroupKind::Counter(metrics) => format_simple_metric(&group.name, metrics),
        GroupKind::Untyped(metrics) => format_simple_metric(&group.name, metrics),
        GroupKind::Summary(metrics) => format_summary_metric(&group.name, metrics),
        GroupKind::Histogram(metrics) => format_histogram_metric(&group.name, metrics),
    }
}

pub fn format_simple_metric(
    group_name: &str,
    metrics: &IndexMap<GroupKey, SimpleMetric>,
) -> String {
    let mut result = String::new();
    for (key, metric) in metrics {
        let timestamp = match key.timestamp {
            Some(ts) => format!(" {}", ts),
            None => String::new(),
        };
        result.push_str(&format!(
            "{}{{{}}} {}{}\n",
            group_name,
            format_labels(&key.labels),
            metric.value,
            timestamp
        ))
    }
    result
}

pub fn format_summary_metric(
    group_name: &str,
    metrics: &IndexMap<GroupKey, SummaryMetric>,
) -> String {
    let mut result = String::new();
    for (key, metric) in metrics {
        let timestamp = match key.timestamp {
            Some(ts) => format!(" {}", ts),
            None => String::new(),
        };
        
        for quantile in &metric.quantiles {
            let mut labels = key.labels.clone();
            labels.insert("quantile".to_string(), quantile.quantile.to_string());
            result.push_str(&format!(
                "{}{{{}}} {}{}\n",
                group_name,
                format_labels(&labels),
                quantile.value,
                timestamp
            ));
        }
        
        let sum_labels = key.labels.clone();
        result.push_str(&format!(
            "{}_sum{{{}}} {}{}\n",
            group_name,
            format_labels(&sum_labels),
            metric.sum,
            timestamp
        ));
        
        let count_labels = key.labels.clone();
        result.push_str(&format!(
            "{}_count{{{}}} {}{}\n",
            group_name,
            format_labels(&count_labels),
            metric.count,
            timestamp
        ));
    }
    result
}

pub fn format_histogram_metric(
    group_name: &str,
    metrics: &IndexMap<GroupKey, HistogramMetric>,
) -> String {
    let mut result = String::new();
    for (key, metric) in metrics {
        let timestamp = match key.timestamp {
            Some(ts) => format!(" {}", ts),
            None => String::new(),
        };
        
        for bucket in &metric.buckets {
            let mut labels = key.labels.clone();
            labels.insert("le".to_string(), bucket.bucket.to_string());
            result.push_str(&format!(
                "{}_bucket{{{}}} {}{}\n",
                group_name,
                format_labels(&labels),
                bucket.count,
                timestamp
            ));
        }
        
        let sum_labels = key.labels.clone();
        result.push_str(&format!(
            "{}_sum{{{}}} {}{}\n",
            group_name,
            format_labels(&sum_labels),
            metric.sum,
            timestamp
        ));
        
        let count_labels = key.labels.clone();
        result.push_str(&format!(
            "{}_count{{{}}} {}{}\n",
            group_name,
            format_labels(&count_labels),
            metric.count,
            timestamp
        ));
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
