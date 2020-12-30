use crate::collectors::hiccups_collector::hiccup_settings::HiccupsMonitorSettings;
use crate::exporters::prometheus_exporter::prometheus_settings::PrometheusSettings;
use crate::metrics::measurement_unit::MEASUREMENT_UNITS;
use crate::metrics::measurement_unit::MeasurementUnit;

pub mod config_loader;

#[derive(Debug, Deserialize, Clone, AsStaticStr)]
pub enum TimeUnitsSettings {
    TimeNanos,
    TimeMicros,
    TimeMillis,
    TimeSeconds,
    InfoBytes,
    InfoKiloBytes,
    InfoMegaBytes,
    InfoGigaBytes,
    Percentage,
    None,
}

impl TimeUnitsSettings {
    pub fn to_measurement_units(&self) -> &'static MeasurementUnit {
        match self {
            TimeUnitsSettings::TimeNanos => &MEASUREMENT_UNITS.time.nanos,
            TimeUnitsSettings::TimeMicros => &MEASUREMENT_UNITS.time.micros,
            TimeUnitsSettings::TimeMillis => &MEASUREMENT_UNITS.time.millis,
            TimeUnitsSettings::TimeSeconds => &MEASUREMENT_UNITS.time.seconds,
            TimeUnitsSettings::InfoBytes => &MEASUREMENT_UNITS.information.bytes,
            TimeUnitsSettings::InfoKiloBytes => &MEASUREMENT_UNITS.information.kilobytes,
            TimeUnitsSettings::InfoMegaBytes => &MEASUREMENT_UNITS.information.megabytes,
            TimeUnitsSettings::InfoGigaBytes => &MEASUREMENT_UNITS.information.gigabytes,
            TimeUnitsSettings::Percentage => &MEASUREMENT_UNITS.percentage,
            TimeUnitsSettings::None => &MEASUREMENT_UNITS.none,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub debug: bool,
    pub prometheus_exporter: PrometheusSettings,
    pub hiccups_monitor: HiccupsMonitorSettings,
}

impl Settings {
    pub fn load() -> Self {
        let s = config_loader::load_config();
        let settings = s.try_into().unwrap();
        info!("Settings: {:?}", settings);
        settings
    }
}
