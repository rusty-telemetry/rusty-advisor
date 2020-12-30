use std::env;

use config::{Config, Environment, File};
use config::Source;

use crate::collectors::hiccups_collector::hiccup_settings::HiccupsMonitorSettings;
use crate::exporters::prometheus_exporter::prometheus_settings::PrometheusSettings;
use crate::strum::AsStaticRef;

pub fn load_config() -> Config {
    let mut config = Config::new();

    add_default_config(&mut config);

    env::var("RUSTY_CONFIG_FILE")
        .map(|config_file| {
            info!("Loading configuration file {}", config_file);
            config.merge(File::with_name(&config_file[..])).unwrap();
            info!("Picking up configs from Env Variables -> Config File -> Defaults");
        })
        .unwrap_or_else(|e| {
            warn!("Config file not provided. You can provide one via the environment variable CONFIG_FILE. (Reason: {})", e);
            info!("Picking up configs from Env Variables -> Defaults");
        });

    // Override any setting from the environment variables (with a prefix of RUSTY)
    // Eg.. `RUSTY_DEBUG=1 ./target/app` would set the `debug` key
    config.merge(Environment::with_prefix("rusty")).unwrap();

    // Now that we're done, let's access our configuration
    info!("Debug: {:?}", config.get_bool("debug"));
    debug!("Provided settings:  {:?}", config.collect());

    config
}

fn add_default_config(config: &mut Config) {
    config.set_default("debug", false).unwrap();
    let prometheus_settings_default = PrometheusSettings::default();
    let hiccups_monitor_default = HiccupsMonitorSettings::default();
    config.set_default("prometheus_exporter.host", prometheus_settings_default.host).unwrap();
    config.set_default("prometheus_exporter.port", prometheus_settings_default.port as i64).unwrap();
    config.set_default("prometheus_exporter.path", prometheus_settings_default.path).unwrap();
    config.set_default("prometheus_exporter.metrics.histograms.buckets.default", prometheus_settings_default.metrics.histograms.buckets.default).unwrap();
    config.set_default("prometheus_exporter.metrics.histograms.buckets.custom_buckets", prometheus_settings_default.metrics.histograms.buckets.custom_buckets).unwrap();
    config.set_default("hiccups_monitor.name", hiccups_monitor_default.name).unwrap();
    config.set_default("hiccups_monitor.description", hiccups_monitor_default.description).unwrap();
    config.set_default("hiccups_monitor.resolution_nanos", hiccups_monitor_default.resolution_nanos as i64).unwrap();
    config.set_default("hiccups_monitor.histogram_settings.min", hiccups_monitor_default.histogram_settings.min as i64).unwrap();
    config.set_default("hiccups_monitor.histogram_settings.max", hiccups_monitor_default.histogram_settings.max as i64).unwrap();
    config.set_default("hiccups_monitor.histogram_settings.precision", hiccups_monitor_default.histogram_settings.precision as i64).unwrap();
    config.set_default("hiccups_monitor.histogram_settings.unit", hiccups_monitor_default.histogram_settings.unit.as_static()).unwrap();
}
