use std::env;
use std::path::PathBuf;

use config::{Config, Environment, File};
use config::Source;

use serde;
use serde::Deserialize;

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
        let mut s = load_config();

        // You can deserialize (and thus freeze) the entire configuration as
        let settings = s.deserialize().unwrap();
        info!("Settings: {:?}", settings);
        settings
    }
}

fn load_config() -> Config {
    let mut config = Config::new();

    config.merge(File::with_name("config/default")).unwrap();

    let env = env::var("RUN_MODE").unwrap_or("dev".into());
    config.merge(File::with_name(&format!("config/{}", env)).required(false)).unwrap();

    // This file shouldn't be checked in to git
    config.merge(File::with_name("config/local").required(false)).unwrap();

    // Override any setting from the environment variables (with a prefix of RUSTY)
    // Eg.. `RUSTY_DEBUG=1 ./target/app` would set the `debug` key
    config.merge(Environment::with_prefix("rusty")).unwrap();

    // Now that we're done, let's access our configuration
    info!("Debug: {:?}", config.get_bool("debug"));
    debug!("Provided settings:  {:?}", config.collect());

    config
}
