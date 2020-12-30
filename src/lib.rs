#![feature(rustc_private)]

extern crate crossbeam_channel;
#[cfg(test)]
#[macro_use]
extern crate float_cmp;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[cfg(test)]
#[macro_use]
extern crate maplit;
extern crate pretty_env_logger;
#[macro_use]
extern crate prometheus;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate strum;
#[macro_use]
extern crate strum_macros;
extern crate tokio;
#[cfg(test)]
extern crate tokio_test;
extern crate toml;

use std::sync::Arc;

use futures::future::join3;
use tokio::runtime;
use tokio::sync::broadcast;

use collectors::hiccups_collector::hiccup_monitor::HiccupMonitor;
use settings::Settings;

use crate::exporters::metrics_exporter::{MetricsExporter, MetricsSnapshot};
use crate::exporters::prometheus_exporter::prometheus_reporter::PrometheusExporter;

pub mod errors;
pub mod settings;
pub mod metrics;
pub mod utils;
mod exporters;
mod collectors;

pub struct RustyAdvisor;


impl RustyAdvisor {
    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        info!("RustyAdvisor is starting...");
        let settings = Settings::load();

        let mut threaded_rt = runtime::Builder::new()
            .threaded_scheduler()
            .enable_all()
            .build()?;

        let (sender, receiver) = broadcast::channel::<Arc<MetricsSnapshot>>(16);

        let mut metrics_exporter = MetricsExporter::new("Global".into());
        let metrics_exporter_ticker = metrics_exporter.start(sender);

        let mut monitor = HiccupMonitor::new(&settings.hiccups_monitor);
        monitor.run();

        let prometheus_exporter = PrometheusExporter::new(settings.prometheus_exporter);
        let prometheus_runtime = prometheus_exporter.start_server();
        let prometheus_listener = prometheus_exporter.listen_metrics(receiver);

        threaded_rt.block_on(join3(metrics_exporter_ticker, prometheus_runtime, prometheus_listener)).0?;
        info!("RustyAdvisor is ending...");
        Ok(())
    }
}
