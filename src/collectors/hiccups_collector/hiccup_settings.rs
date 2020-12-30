use crate::settings::TimeUnitsSettings;

#[derive(Debug, Deserialize, Clone)]
pub struct HiccupsMonitorSettings {
    pub name: String,
    pub description: String,
    pub resolution_nanos: u64,
    pub histogram_settings: HiccupsHistogramSettings,
}

#[derive(Debug, Deserialize, Clone)]
pub struct HiccupsHistogramSettings {
    pub min: u64,
    pub max: u64,
    pub precision: u8,
    pub unit: TimeUnitsSettings,
}

impl Default for HiccupsMonitorSettings {
    fn default() -> Self {
        HiccupsMonitorSettings {
            name: "hiccups_duration_seconds".into(),
            description: "Hiccups detected in the VM expressed in nanoseconds.".into(),
            resolution_nanos: 1_000_000,
            histogram_settings: HiccupsHistogramSettings::default(),
        }
    }
}

impl Default for HiccupsHistogramSettings {
    fn default() -> Self {
        HiccupsHistogramSettings {
            min: 1,
            max: 1_000_000,
            precision: 0,
            unit: TimeUnitsSettings::TimeNanos,
        }
    }
}
