#![feature(rustc_private)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate prometheus;

mod exporters;
mod collectors;

use exporters::prometheus_exporter::PrometheusExporter;
use collectors::hiccups_collector::hiccup_monitor::HiccupMonitor;

pub struct RustyAdvisor;

impl RustyAdvisor {

    pub fn run() {
        println!("RustyAdvisor is starting...");
        let mut monitor = HiccupMonitor::new();
        monitor.run();

        PrometheusExporter::start_up();
        println!("RustyAdvisor is ending...");
    }
}
