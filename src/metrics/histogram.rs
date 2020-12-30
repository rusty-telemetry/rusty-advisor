use std::{fmt, time};
use std::collections::HashMap;
use std::fmt::Display;

use hdrhistogram::{Histogram as HdrHistogram, SyncHistogram};
use hdrhistogram::sync::Recorder;
use serde::export::Formatter;
use tokio::time::{Duration, Instant};

use crate::errors::{Error, Result};
use crate::exporters::metrics_exporter::HistogramSample;
use crate::metrics::{measurement_unit, registry};
use crate::metrics::measurement_unit::{MEASUREMENT_UNITS, MeasurementUnit};
use crate::metrics::metric::MetricDescription;

#[derive(Clone, Debug)]
pub struct HistogramBuilder {
    pub name: String,
    pub description: String,
    pub tags: HashMap<String, String>,
    pub settings: HistogramSettings,
}

impl HistogramBuilder {
    pub fn new(name: String, description: String) -> HistogramBuilder {
        HistogramBuilder {
            name,
            description,
            tags: HashMap::new(),
            settings: HistogramSettings::default(),
        }
    }

    pub fn with_tags(mut self, name: String, value: String) -> HistogramBuilder {
        self.tags.insert(name, value);
        self
    }

    pub fn with_settings(mut self, histogram_settings: HistogramSettings) -> HistogramBuilder {
        self.settings = histogram_settings;
        self
    }

    pub fn metric_description(&self) -> Result<MetricDescription> {
        MetricDescription::from(self.name.clone(), self.description.clone(), self.tags.clone())
    }

    pub async fn build(self) -> Result<HistogramRecorder> {
        registry::global_registry().get_or_register_histogram(self).await
    }

    /// build_sync has to be used when the caller is running out of the Tokio async runtime
    #[tokio::main]
    pub async fn build_sync(self) -> Result<HistogramRecorder> {
        registry::global_registry().get_or_register_histogram(self).await
    }
}

#[derive(Debug)]
pub struct HistogramRecorder {
    recorder: Recorder<u64>,
    pub measurement_unit: &'static MeasurementUnit,
}

impl HistogramRecorder {
    pub fn new(recorder: Recorder<u64>, measurement_unit: &'static MeasurementUnit) -> HistogramRecorder {
        HistogramRecorder {
            recorder,
            measurement_unit,
        }
    }

    pub fn record(&mut self, value: u64) -> Result<()> {
        self.recorder.record(value)
            .map_err(|error| { Error::Msg(format!("Error occurs trying to record value {} on a histogram. Reason: {:#?}", value, error)) })
    }

    pub fn record_duration(&mut self, duration: Duration) -> Result<()> {
        let value = measurement_unit::convert(duration.as_secs_f64(), &MEASUREMENT_UNITS.time.seconds, self.measurement_unit) as u64;
        self.recorder.record(value)
            .map_err(|error| { Error::Msg(format!("Error occurs trying to record value {} on a histogram. Reason: {:#?}", value, error)) })
    }

    pub fn start_timer(&mut self) -> HistogramTimer {
        HistogramTimer::new(self)
    }
}

pub struct HistogramTimer<'a> {
    recorder: &'a mut HistogramRecorder,
    start: Instant,
}

impl<'a> HistogramTimer<'a> {
    fn new(recorder: &'a mut HistogramRecorder) -> HistogramTimer {
        HistogramTimer {
            recorder,
            start: Instant::now(),
        }
    }

    pub fn close(&mut self) -> Duration {
        let duration = self.start.elapsed();
        debug!("Histogram timer samples duration {} millis", duration.as_millis());
        self.recorder.record_duration(duration);
        duration
    }
}

#[derive(Clone, Debug)]
pub struct HistogramSettings {
    pub low: u64,
    pub high: u64,
    pub precision: u8,
    pub measurement_unit: &'static MeasurementUnit,
}

impl HistogramSettings {
    pub fn from(low: u64, high: u64, precision: u8, measurement_unit: &'static MeasurementUnit) -> HistogramSettings {
        HistogramSettings {
            low,
            high,
            precision,
            measurement_unit,
        }
    }
}

impl Default for HistogramSettings {
    fn default() -> Self {
        HistogramSettings {
            low: 1,
            high: 1_000_000,
            precision: 2,
            measurement_unit: &MEASUREMENT_UNITS.time.seconds,
        }
    }
}

impl Display for HistogramSettings {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "[low: {}, high: {}, presicion: {}. {}]", self.low, self.high, self.precision, self.measurement_unit)
    }
}

#[derive(Debug)]
pub struct Histogram {
    metric_description: MetricDescription,
    histogram_settings: HistogramSettings,
    hdr_histogram: SyncHistogram<u64>,
}

impl Histogram {
    pub fn new(metric_description: MetricDescription, histogram_settings: HistogramSettings) -> Result<Histogram> {
        HdrHistogram::<u64>::new_with_bounds(histogram_settings.low, histogram_settings.high, histogram_settings.precision)
            .map_err(|error| Error::Msg(format!("Error creating Histogram. Reason: {}", error.to_string())))
            .map(|hdr_histogram|
                Histogram {
                    metric_description,
                    histogram_settings,
                    hdr_histogram: hdr_histogram.into_sync(),
                })
    }

    /// This method is not thread safe
    pub fn sample(&mut self, reset: bool) -> HistogramSample {
        self.hdr_histogram.refresh_timeout(time::Duration::from_millis(1));
        let histogram_sample = self.hdr_histogram.clone_correct(self.hdr_histogram.max());
        if reset {
            self.hdr_histogram.reset();
        }
        HistogramSample::new(histogram_sample, self.histogram_settings.clone())
    }

    pub fn new_recorder(&self) -> HistogramRecorder {
        HistogramRecorder::new(self.hdr_histogram.recorder(), self.histogram_settings.measurement_unit)
    }

    pub fn metric_description(&self) -> &MetricDescription {
        &self.metric_description
    }
}
