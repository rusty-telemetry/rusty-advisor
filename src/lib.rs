#![feature(rustc_private)]

#[macro_use]
extern crate log;
extern crate pretty_env_logger;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate toml;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate prometheus;

mod exporters;
mod collectors;
pub mod settings;

use exporters::prometheus_exporter::PrometheusExporter;
use collectors::hiccups_collector::hiccup_monitor::HiccupMonitor;
use settings::Settings;

pub struct RustyAdvisor;


impl RustyAdvisor {

    pub fn run() {
        info!("RustyAdvisor is starting...");
        let settings = Settings::load();
        let mut monitor = HiccupMonitor::new(& settings.hiccups_monitor);
        monitor.run();

        PrometheusExporter::start_up(settings);
        info!("RustyAdvisor is ending...");
    }
}
