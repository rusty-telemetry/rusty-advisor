use std::env;
use std::path::PathBuf;

use config::{Config, Environment, File};
use config::Source;

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
    config.set_default("prometheus_exporter.host", "0.0.0.0").unwrap();
    config.set_default("prometheus_exporter.port", 9095).unwrap();
    config.set_default("prometheus_exporter.path", "/metrics").unwrap();
    config.set_default("hiccups_monitor.resolution_nanos", 100).unwrap();
}
