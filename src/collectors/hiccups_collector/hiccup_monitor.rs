use std::{sync, thread};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use hdrhistogram::Histogram;

pub struct HiccupMonitor {
    hiccup_nanos: u64,
    histogram: Arc<Mutex<Histogram<u64>>>,
    handle: Option<thread::JoinHandle<()>>,
    running: sync::Arc<AtomicBool>
}

impl HiccupMonitor {
    pub fn new() -> HiccupMonitor {
        HiccupMonitor {
            hiccup_nanos: 100,
            histogram: Arc::new(Mutex::new(Histogram::<u64>::new(2).unwrap())),
            running: sync::Arc::new(AtomicBool::new(true)),
            handle: None
        }
    }

    pub fn run(&mut self) {
        let shortest_observed_delta = std::u64::MAX;
        let resolution = self.hiccup_nanos.clone();
        let is_running = self.running.clone();
        let histogram : Arc<Mutex<Histogram<u64>>> = self.histogram.clone();

        self.handle = Some(thread::Builder::new().name("hiccup-monitor".into()).spawn(move || {
            while is_running.load(Ordering::SeqCst) {
                let hiccup_time = hicc(resolution, shortest_observed_delta);
                record(histogram.clone(),hiccup_time, resolution);
            }
        }).unwrap());

        fn hicc(resolution: u64, mut shortest_observed_delta: u64) -> u64 {
            let start = Instant::now();
            thread::sleep(Duration::from_nanos(resolution));
            let delta = start.elapsed().as_nanos() as u64;
            if delta < shortest_observed_delta { shortest_observed_delta = delta }
            // delta - shortest_observed_delta
            delta
        }

        /// We'll need fill in missing measurements as delayed
        fn record(histogram : Arc<Mutex<Histogram<u64>>>, value: u64, expected_interval_between_value_samples: u64) {
            histogram.lock().unwrap().record(value).unwrap();
            if expected_interval_between_value_samples > 0 {
                let mut missing_value = value - expected_interval_between_value_samples;
                while missing_value >= expected_interval_between_value_samples {
                    histogram.lock().unwrap().record(missing_value).unwrap();
                    missing_value -= expected_interval_between_value_samples
                }
            }
        }
    }

    pub fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        self.handle
            .take().expect("Called stop")
            .join().expect("Could not join spawned thread");
    }

    /// testing
    pub fn print(&mut self) {
        println!("# of samples: {}", self.histogram.lock().unwrap().len());
        println!("99.9'th percentile: {}", self.histogram.lock().unwrap().value_at_quantile(0.999));
    }
}
