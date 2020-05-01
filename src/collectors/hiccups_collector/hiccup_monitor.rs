use std::{sync, thread};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use hdrhistogram::Histogram;
use pretty_env_logger::env_logger;
use pretty_env_logger::env_logger::Env;
use prometheus::core::Metric;
use prometheus::HistogramVec;

use crate::settings::HiccupsMonitorConfig;

pub struct HiccupMonitor {
    hiccup_nanos: u64,
    // histogram: Arc<Mutex<Histogram<u64>>>,
    histogram: Arc<Mutex<HistogramVec>>,
    handle: Option<thread::JoinHandle<()>>,
    running: sync::Arc<AtomicBool>,
}

impl HiccupMonitor {
    pub fn new(config: &HiccupsMonitorConfig) -> HiccupMonitor {
        HiccupMonitor {
            hiccup_nanos: config.resolution_nanos,
            // histogram: Arc::new(Mutex::new(Histogram::<u64>::new(2).unwrap())),
            histogram: Arc::new(Mutex::new(register_histogram_vec!(
                "hiccups_duration_seconds",
                "hiccups detected in the VM expressed in nanoseconds.",
                &["handler"])
                .unwrap())),
            running: sync::Arc::new(AtomicBool::new(true)),
            handle: None,
        }
    }

    pub fn run(&mut self) {
        info!("Hiccups Monitor running...");

        let mut shortest_observed_delta = std::u64::MAX;
        let resolution = self.hiccup_nanos.clone();
        let is_running = self.running.clone();
        let histogram: Arc<Mutex<HistogramVec>> = self.histogram.clone();

        self.handle = Some(thread::Builder::new().name("hiccup-monitor".into()).spawn(move || {
            while is_running.load(Ordering::SeqCst) {
                let hiccup_time = hicc(resolution, &mut shortest_observed_delta);
                record(histogram.clone(), hiccup_time, resolution);
            }
        }).unwrap());

        fn hicc(resolution: u64, shortest_observed_delta: &mut u64) -> u64 {
            let start = Instant::now();
            thread::sleep(Duration::from_nanos(resolution));
            let delta = start.elapsed().as_nanos() as u64;
            if delta < *shortest_observed_delta { *shortest_observed_delta = delta }
            delta - *shortest_observed_delta
        }

        /// We'll need fill in missing measurements as delayed
        fn record(histogram: Arc<Mutex<HistogramVec>>, value: u64, expected_interval_between_value_samples: u64) {
            // histogram.lock().unwrap().record(value).unwrap();
            histogram.lock().unwrap().with_label_values(&["all"]).observe(value as f64);
            if expected_interval_between_value_samples > 0 {
                let mut missing_value = if let Some(v) = value.checked_sub(expected_interval_between_value_samples) { v } else { 0 };
                while missing_value >= expected_interval_between_value_samples {
                    // histogram.lock().unwrap().record(missing_value).unwrap();
                    histogram.lock().unwrap().with_label_values(&["all"]).observe(missing_value as f64);
                    missing_value -= expected_interval_between_value_samples
                }
            }
        }
    }

    pub fn stop(&mut self) {
        info!("Hiccups Monitor stopping...");
        self.running.store(false, Ordering::SeqCst);
        self.handle
            .take().expect("Called stop")
            .join().expect("Could not join spawned thread");
    }

    /// testing
    pub fn print(&mut self) {
        // println!("# of samples: {}", self.histogram.lock().unwrap().len());
        println!("# of samples: {}", self.histogram.lock().unwrap().with_label_values(&["all"]).get_sample_count());
        // println!("99.9'th percentile: {}", self.histogram.lock().unwrap().with_label_values(&["all"]).metric());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_working() {
        let config = HiccupsMonitorConfig { resolution_nanos: 1000 };
        let mut monitor = HiccupMonitor::new(&config);

        monitor.run();

        thread::sleep(Duration::from_millis(5000));

        monitor.print();

        monitor.stop()
    }
}
