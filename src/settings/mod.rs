use serde;
use serde::Deserialize;

pub mod config_loader;

#[derive(Debug, Deserialize, Clone)]
pub struct PrometheusExporterConfig {
    pub host: String,
    pub port: u16,
    pub path: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct HiccupsMonitorConfig {
    pub resolution_nanos: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub debug: bool,
    pub prometheus_exporter: PrometheusExporterConfig,
    pub hiccups_monitor: HiccupsMonitorConfig,
}

impl Settings {
    pub fn load() -> Self {
        let mut s = config_loader::load_config();

        // You can deserialize (and thus freeze) the entire configuration as
        let settings = s.deserialize().unwrap();
        info!("Settings: {:?}", settings);
        settings
    }
}
