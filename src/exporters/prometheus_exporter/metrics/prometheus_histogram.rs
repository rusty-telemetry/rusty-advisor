use std::sync::Arc;
use std::time::Instant;

use crate::exporters::metrics_exporter::HistogramSample;
use crate::exporters::prometheus_exporter::prometheus_settings::{PrometheusHistogramSettings, PrometheusSettings};
use crate::metrics::measurement_unit;
use crate::metrics::measurement_unit::MEASUREMENT_UNITS;
use crate::metrics::metric::MetricDescription;
use crate::prometheus::core::Number;
use crate::utils::time;

type BucketHolder = Vec<(f64, u64)>;

#[derive(Debug)]
pub struct PrometheusHistogram {
    metric_description: Arc<MetricDescription>,
    buckets: BucketHolder,
    count: u64,
    sum: f64,
    timestamp_ms: u64,
}

impl PrometheusHistogram {
    pub fn new(metric_description: Arc<MetricDescription>, settings: PrometheusSettings) -> Self {
        let buckets = Self::create_buckets(&metric_description, &settings.metrics.histograms);
        PrometheusHistogram {
            metric_description,
            buckets,
            count: 0,
            sum: 0 as f64,
            timestamp_ms: time::current_millis(),
        }
    }

    fn create_buckets(metric_description: &Arc<MetricDescription>, histo_settings: &PrometheusHistogramSettings) -> BucketHolder {
        let buckets = histo_settings.buckets.from(&metric_description.name).clone();
        let mut buckets_holder = Vec::<(f64, u64)>::with_capacity(buckets.len() + 1);
        for bucket in &buckets {
            buckets_holder.push((bucket.into_f64(), 0 as u64));
        }
        buckets_holder.push((f64::MAX, 0 as u64));
        buckets_holder
    }

    /// # Description
    ///
    /// *This function is as awful as efficient.*
    ///
    /// Takes O(n) to add a Histogram Sample to the Prometheus Histogram
    ///
    /// # Example
    ///
    /// Given:
    /// - Prometheus Buckets = `[2,4,6,8,10]`
    /// - Prometheus Histogram = `((2,0),(4,0),(6,0),(8,0),(10,0),(Inf,0))`
    /// - Histogram Sample = `((0,3),(1,5),(3,1),(5,10),(6,7))`
    ///
    /// The Prometheus histogram should end with the following values:
    /// - Prometheus Histogram = `((2,8),(4,9),(6,26),(8,26),(10,26),(Inf,26))`
    ///
    /// For more example take a look to the unit tests.
    pub fn add_snapshot(&mut self, histogram_sample: &HistogramSample, timestamp_in_millis: u64) {
        if self.buckets.len() <= 0 {
            return;
        }
        let start = Instant::now();

        let mut next_bucket_index = 0;
        let mut next_bucket = self.buckets[next_bucket_index].0;
        let mut sum_samples = 0 as f64;
        let mut count_samples = 0 as u64;

        let hdr_histogram = histogram_sample.hdr_histogram();

        for record in hdr_histogram.iter_recorded() {
            let value = measurement_unit::convert(record.value_iterated_to() as f64, histogram_sample.measurement_unit(), &MEASUREMENT_UNITS.time.seconds);
            let count = record.count_at_value();

            while value > next_bucket && next_bucket_index <= self.buckets.len() - 1 {
                self.buckets[next_bucket_index].1 += count_samples;
                next_bucket_index += 1;
                next_bucket = self.buckets[next_bucket_index].0;
            }

            sum_samples += value * count as f64;
            count_samples += count;
        }

        for remaining_bucket in &mut self.buckets[next_bucket_index..] {
            remaining_bucket.1 += count_samples;
        }

        self.sum += sum_samples;
        self.count += count_samples;

        self.timestamp_ms = timestamp_in_millis;

        let delta = start.elapsed().as_millis() as u64;
        info!("Inserted {} values on prometheus histogram in {} millis", hdr_histogram.len(), delta);
    }

    pub fn metric_description(&self) -> &MetricDescription {
        &self.metric_description
    }

    pub fn buckets(&self) -> &BucketHolder {
        &self.buckets
    }

    pub fn sum(&self) -> f64 {
        self.sum
    }

    pub fn count(&self) -> u64 {
        self.count
    }

    pub fn timestamp_ms(&self) -> u64 {
        self.timestamp_ms
    }
}


#[cfg(test)]
mod tests {
    use hdrhistogram::Histogram as HdrHistogram;

    use crate::metrics::histogram::HistogramSettings;
    use crate::utils::tests::ApproxComparison;

    use super::*;

    lazy_static! {
        static ref DEFAULTS: Defaults = Defaults::new();
    }

    struct Defaults {
        metric_description: MetricDescription,
        timestamp_in_millis: u64,
    }

    impl Defaults {
        fn new() -> Defaults {
            Defaults {
                metric_description: MetricDescription::from("metric_name_1".to_string(), "some description".to_string(), hashmap! {"tag_1".into() => "tag_value_1".into(), "tag_2".into() => "tag_value_2".into()}).unwrap(),
                timestamp_in_millis: time::current_millis(),
            }
        }
    }

    #[test]
    fn test_translate_histogram_snapshot_with_zeros_to_prometheus_histogram() {
        let metric_description = DEFAULTS.metric_description.clone();
        let histogram_settings = HistogramSettings::from(1, 10, 0, &MEASUREMENT_UNITS.time.nanos);
        let hdr_histogram = HdrHistogram::<u64>::new_with_bounds(histogram_settings.low, histogram_settings.high, histogram_settings.precision)
            .unwrap();
        let histogram_sample = HistogramSample::new(hdr_histogram, histogram_settings.clone());
        let mut prometheus_histogram = PrometheusHistogram::new(Arc::new(metric_description), PrometheusSettings::default());
        prometheus_histogram.add_snapshot(&histogram_sample, DEFAULTS.timestamp_in_millis);

        assert_eq!(prometheus_histogram.buckets.len(), 10);
        for bucket in prometheus_histogram.buckets {
            assert_eq!(bucket.1, 0);
        }
    }

    #[test]
    fn test_add_snapshot_with_one_tick_records_to_prometheus_histogram() {
        let mut settings = PrometheusSettings::default();
        settings.metrics.histograms.buckets.default = vec![
            2f64, 4f64, 6f64, 8f64, 10f64,
        ];
        let metric_description = DEFAULTS.metric_description.clone();
        let histogram_settings = HistogramSettings::from(1, 1000, 2, &MEASUREMENT_UNITS.time.seconds);
        let mut hdr_histogram = HdrHistogram::<u64>::new_with_bounds(histogram_settings.low, histogram_settings.high, histogram_settings.precision)
            .unwrap();
        hdr_histogram.record_n(0, 3);
        hdr_histogram.record_n(1, 5);
        hdr_histogram.record_n(3, 1);
        hdr_histogram.record_n(5, 10);
        hdr_histogram.record_n(6, 7);
        let histogram_sample = HistogramSample::new(hdr_histogram, histogram_settings.clone());
        let mut prometheus_histogram = PrometheusHistogram::new(Arc::new(metric_description), settings);
        prometheus_histogram.add_snapshot(&histogram_sample, DEFAULTS.timestamp_in_millis);

        assert_eq!(prometheus_histogram.buckets.len(), 6);
        assert_eq!(prometheus_histogram.buckets[0].1, 8);  // 2
        assert_eq!(prometheus_histogram.buckets[1].1, 9);  // 4
        assert_eq!(prometheus_histogram.buckets[2].1, 26); // 6
        assert_eq!(prometheus_histogram.buckets[3].1, 26); // 8
        assert_eq!(prometheus_histogram.buckets[4].1, 26); // 10
        assert_eq!(prometheus_histogram.buckets[5].1, 26); // Inf
        assert_eq!(prometheus_histogram.count, 26);
        assert!(prometheus_histogram.sum.is_eq(100 as f64, 0i64));
    }

    #[test]
    fn test_add_multiple_snapshots_with_new_records_to_prometheus_histogram() {
        let mut settings = PrometheusSettings::default();
        settings.metrics.histograms.buckets.default = vec![
            2f64, 4f64, 6f64, 8f64, 10f64,
        ];
        let metric_description = DEFAULTS.metric_description.clone();
        let mut prometheus_histogram = PrometheusHistogram::new(Arc::new(metric_description), settings);

        let histogram_settings = HistogramSettings::from(1, 1000, 2, &MEASUREMENT_UNITS.time.seconds);
        let mut hdr_histogram = HdrHistogram::<u64>::new_with_bounds(histogram_settings.low, histogram_settings.high, histogram_settings.precision)
            .unwrap();
        hdr_histogram.record_n(0, 3);
        hdr_histogram.record_n(1, 5);
        hdr_histogram.record_n(3, 1);
        hdr_histogram.record_n(5, 10);
        hdr_histogram.record_n(6, 7);
        let histogram_sample = HistogramSample::new(hdr_histogram.clone(), histogram_settings.clone());
        prometheus_histogram.add_snapshot(&histogram_sample, DEFAULTS.timestamp_in_millis);

        hdr_histogram.reset();
        hdr_histogram.record_n(0, 3);
        hdr_histogram.record_n(1, 20);
        hdr_histogram.record_n(3, 13);
        hdr_histogram.record_n(4, 45);
        hdr_histogram.record_n(5, 71);
        hdr_histogram.record_n(6, 51);
        hdr_histogram.record_n(7, 27);
        hdr_histogram.record_n(8, 35);
        hdr_histogram.record_n(9, 115);
        hdr_histogram.record_n(12, 23);
        let histogram_sample = HistogramSample::new(hdr_histogram.clone(), histogram_settings.clone());
        prometheus_histogram.add_snapshot(&histogram_sample, DEFAULTS.timestamp_in_millis);

        assert_eq!(prometheus_histogram.buckets.len(), 6);
        assert_eq!(prometheus_histogram.buckets[0].1, 31);  // 2
        assert_eq!(prometheus_histogram.buckets[1].1, 90);  // 4
        assert_eq!(prometheus_histogram.buckets[2].1, 229); // 6
        assert_eq!(prometheus_histogram.buckets[3].1, 291); // 8
        assert_eq!(prometheus_histogram.buckets[4].1, 406); // 10
        assert_eq!(prometheus_histogram.buckets[5].1, 429); // Inf
        assert_eq!(prometheus_histogram.count, 429);
        assert!(prometheus_histogram.sum.is_eq(2780 as f64, 0i64));
    }
}
