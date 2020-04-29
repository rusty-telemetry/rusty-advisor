#![feature(rustc_private)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate prometheus;

mod exporters;
mod collectors;

use exporters::prometheus_exporter::PrometheusExporter;

pub struct RustyAdvisor;

impl RustyAdvisor {

    pub fn run() {
        println!("RustyAdvisor is starting...");

        PrometheusExporter::startUp();
        println!("RustyAdvisor is ending...");
    }
}
