use std::{sync, thread};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use crate::collectors::hiccups_collector::hiccup_settings::HiccupsMonitorSettings;
use crate::metrics::histogram::{HistogramBuilder, HistogramRecorder, HistogramSettings};

pub struct HiccupMonitor {
    hiccup_nanos: u64,
    histogram: Arc<Mutex<HistogramRecorder>>,
    handle: Option<thread::JoinHandle<()>>,
    running: sync::Arc<AtomicBool>,
}

impl HiccupMonitor {
    pub fn new(config: &HiccupsMonitorSettings) -> HiccupMonitor {
        info!("Starting Hiccups-Monitor [resolution = {} nanos]", config.resolution_nanos);
        let histogram_publisher = HistogramBuilder::new(config.name.clone(), config.description.clone())
            .with_tags("component".to_string(), "rusty_advisor".to_string())
            .with_settings(HistogramSettings::from(config.histogram_settings.min, config.histogram_settings.max, config.histogram_settings.precision, config.histogram_settings.unit.to_measurement_units()))
            .build_sync()
            .unwrap();
        HiccupMonitor {
            hiccup_nanos: config.resolution_nanos,
            histogram: Arc::new(Mutex::new(histogram_publisher)),
            running: sync::Arc::new(AtomicBool::new(true)),
            handle: None,
        }
    }

    pub fn run(&mut self) {
        info!("Hiccups Monitor running...");

        let mut shortest_observed_delta = std::u64::MAX;
        let resolution = self.hiccup_nanos.clone();
        let is_running = self.running.clone();
        let histogram: Arc<Mutex<HistogramRecorder>> = self.histogram.clone();

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
        fn record(histogram: Arc<Mutex<HistogramRecorder>>, value: u64, expected_interval_between_value_samples: u64) {
            histogram.lock().unwrap().record(value);
            if expected_interval_between_value_samples > 0 {
                let mut missing_value = if let Some(v) = value.checked_sub(expected_interval_between_value_samples) { v } else { 0 };
                while missing_value >= expected_interval_between_value_samples {
                    histogram.lock().unwrap().record(missing_value);
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
}

#[cfg(test)]
mod tests {
    use crate::collectors::hiccups_collector::hiccup_settings::{HiccupsHistogramSettings, HiccupsMonitorSettings};

    use super::*;

    #[test]
    fn test_is_working() {
        let config = HiccupsMonitorSettings {
            name: "some_name".to_string(),
            description: "some description".to_string(),
            resolution_nanos: 1000,
            histogram_settings: HiccupsHistogramSettings::default(),
        };
        let mut monitor = HiccupMonitor::new(&config);

        monitor.run();

        thread::sleep(Duration::from_millis(500));

        monitor.stop()
    }
}
