use std::sync::Arc;

use crate::exporters::prometheus_exporter::prometheus_settings::PrometheusSettings;
use crate::metrics::metric::MetricDescription;
use crate::utils::time;

#[derive(Debug)]
pub struct PrometheusCounter {
    metric_description: Arc<MetricDescription>,
    count: u64,
    timestamp_ms: u64,
}

// TODO: implement prometheus counter
impl PrometheusCounter {
    pub fn new(metric_description: Arc<MetricDescription>, _settings: PrometheusSettings) -> Self {
        PrometheusCounter {
            metric_description,
            count: 0,
            timestamp_ms: time::current_millis(),
        }
    }

    /// Insert counter sample values on Prometheus Counter
    pub fn add_snapshot(&mut self, _timestamp_in_millis: u64) {
        unimplemented!()
    }

    pub fn metric_description(&self) -> &MetricDescription {
        &self.metric_description
    }

    pub fn count(&self) -> u64 {
        self.count
    }

    pub fn timestamp_ms(&self) -> u64 {
        self.timestamp_ms
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
// }
