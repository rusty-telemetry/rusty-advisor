# Rusty Advisor
Agent for gathering system metrics and exporting to Prometheus.

### Config

`Rusty-Advisor` can be configured using a config file, specified by environment variable `RUSTY_CONFIG_FILE`.

It can be provided in the following formats:
* [JSON]
* [YAML]
* [TOML]
* [HJSON]
* [INI]

It's possible to override any settings by an environment variable -prefixed with `RUSTY`-.

Every option has a sane default, which means you can run `Rusty-Advisor`
without providing any configuration.

* _Default values in `TOML` format:_
```toml
debug = false

[prometheus_exporter]
host = "0.0.0.0"
port = 9095
path = "/metrics"

[hiccups_monitor]
resolution_nanos = 100
```

* _Example of environment variables:_
```bash
RUSTY_DEBUG=true
RUSTY_PROMETHEUS_EXPORTER.HOST=127.0.0.1
```

### Tests

```bash
 cargo test -- --nocapture
```

[JSON]: https://github.com/serde-rs/json
[TOML]: https://github.com/toml-lang/toml
[YAML]: https://github.com/chyh1990/yaml-rust
[HJSON]: https://github.com/hjson/hjson-rust
[INI]: https://github.com/zonyitoo/rust-ini
