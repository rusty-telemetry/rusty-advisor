use std::ops::DerefMut;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use hdrhistogram::Histogram as HdrHistogram;
use tokio::sync::broadcast::Sender;
use tokio::task::JoinError;

use crate::metrics::histogram::{Histogram, HistogramSettings};
use crate::metrics::measurement_unit::MeasurementUnit;
use crate::metrics::metric::MetricDescription;
use crate::metrics::registry;
use crate::utils::time;

pub struct MetricsExporter {
    name: String,
    running: Arc<AtomicBool>,
}

impl MetricsExporter {
    pub fn new(name: String) -> MetricsExporter {
        MetricsExporter {
            name,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub async fn start(&mut self, sender: Sender<Arc<MetricsSnapshot>>) -> Result<(), JoinError> {
        self.running.store(true, Ordering::SeqCst);
        let is_running = self.running.clone();
        let name = self.name.clone();
        tokio::spawn(async move {
            while is_running.load(Ordering::SeqCst) {
                info!("Executing a tick to publish a metrics snapshot on metric exporter {}", name); // FIXME change by tokio-trace
                let metric_snapshot = Self::tick().await;
                let sending_result = sender.send(Arc::new(metric_snapshot));
                match sending_result {
                    Result::Ok(num_of_receivers) => debug!("Metric snapshot sent to {} receivers", num_of_receivers),
                    Result::Err(_) => debug!("Metric snapshot was not sent to any receiver"),
                }
                tokio::time::delay_for(Duration::from_secs(15)).await;
            }
        }).await
    }

    async fn tick() -> MetricsSnapshot {
        let start = Instant::now();
        let timestamp_in_millis = time::current_millis();
        let metrics = registry::global_registry().histograms();
        let mut samples = Vec::<MetricSample>::with_capacity(metrics.len());
        for metric in metrics {
            let mut mut_metric = metric.write().await;
            samples.push(Self::sample_histograms(mut_metric.deref_mut()));
        }
        let metric_snapshot = MetricsSnapshot::new(samples, timestamp_in_millis);
        let delta = start.elapsed().as_millis() as u64;
        info!("Metric Snapshot created in {} millis", delta);
        metric_snapshot
    }

    fn sample_histograms(histogram: &mut Histogram) -> MetricSample {
        let histogram_sample = histogram.sample(true);
        MetricSample::Histogram(histogram.metric_description().clone(), histogram_sample)
        // unimplemented!()
        // match metric_kind {
        //     MetricKind::Histogram => {
        //         let histogram_sample = histogram.sample(true);
        //         MetricSample::Histogram(metric_description, histogram_sample)
        //     },
        //     MetricKind::Counter => {
        //         unimplemented!()
        //     },
        //     MetricKind::Gauge => {
        //         unimplemented!()
        //
        //     },
        // }
    }
}

#[derive(Default)]
pub struct MetricsSnapshot {
    samples: Vec<MetricSample>,
    timestamp_in_millis: u64,
}

impl MetricsSnapshot {
    fn new(samples: Vec<MetricSample>, timestamp_in_millis: u64) -> MetricsSnapshot {
        MetricsSnapshot {
            samples,
            timestamp_in_millis,
        }
    }

    pub fn samples(&self) -> &Vec<MetricSample> {
        &self.samples
    }

    pub fn timestamp_in_millis(&self) -> u64 {
        self.timestamp_in_millis
    }
}

#[derive(Debug)]
pub enum MetricSample {
    Counter(MetricDescription, CounterSample),
    Gauge(MetricDescription, GaugeSample),
    Histogram(MetricDescription, HistogramSample),
}

#[derive(Debug)]
pub struct CounterSample {
    value: u64,
}

impl CounterSample {
    pub fn new(value: u64) -> CounterSample {
        CounterSample {
            value,
        }
    }
}

#[derive(Debug)]
pub struct GaugeSample {
    value: u64,
}

impl GaugeSample {
    pub fn new(value: u64) -> GaugeSample {
        GaugeSample {
            value,
        }
    }
}

#[derive(Debug)]
pub struct HistogramSample {
    hdr_histogram: HdrHistogram<u64>,
    histogram_settings: HistogramSettings,
}

impl HistogramSample {
    pub fn new(hdr_histogram: HdrHistogram<u64>, histogram_settings: HistogramSettings) -> HistogramSample {
        HistogramSample {
            hdr_histogram,
            histogram_settings,
        }
    }

    pub fn hdr_histogram(&self) -> &HdrHistogram<u64> {
        &self.hdr_histogram
    }

    pub fn histogram_settings(&self) -> &HistogramSettings {
        &self.histogram_settings
    }

    pub fn measurement_unit(&self) -> &'static MeasurementUnit {
        &self.histogram_settings.measurement_unit
    }
}
