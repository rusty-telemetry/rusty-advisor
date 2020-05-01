use pretty_env_logger::env_logger;
use pretty_env_logger::env_logger::Env;

use rusty_advisor::RustyAdvisor;

fn main() {
    env_logger::from_env(Env::default().default_filter_or("info")).init();
    RustyAdvisor::run();
}
