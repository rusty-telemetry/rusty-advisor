use std::collections::HashMap;

pub type BucketName = String;
pub type BucketValues = Vec<f64>;

#[derive(Debug, Deserialize, Clone)]
pub struct PrometheusSettings {
    pub host: String,
    pub port: u16,
    pub path: String,
    pub metrics: PrometheusMetricsSettings,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct PrometheusMetricsSettings {
    pub histograms: PrometheusHistogramSettings,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct PrometheusHistogramSettings {
    pub buckets: Buckets,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Buckets {
    pub default: BucketValues,
    pub custom_buckets: HashMap<BucketName, BucketValues>,
}

impl Buckets {
    pub fn from(&self, name: &String) -> &BucketValues {
        self.custom_buckets.get(name).unwrap_or_else(|| { &self.default })
    }
}

impl Default for PrometheusSettings {
    fn default() -> Self {
        PrometheusSettings {
            host: "0.0.0.0".to_string(),
            port: 9096,
            path: "/metrics".to_string(),
            metrics: PrometheusMetricsSettings::default(),
        }
    }
}


impl Default for Buckets {
    fn default() -> Self {
        let mut custom_buckets = HashMap::<BucketName, BucketValues>::with_capacity(1);
        custom_buckets.insert("hiccups_duration_seconds".to_string(), vec!(
            0.000_000_050, 0.000_000_100, 0.000_000_250, 0.000_000_500, 0.000_001_000, 0.000_002_500, 0.000_005_000, 0.000_010_000, 0.000_025_000, 0.000_050_000, 0.000_100_000,
        ));
        custom_buckets.insert("prometheus_http_request_duration_seconds".to_string(), vec!(
            0.000_5, 0.001, 0.002_5, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.50, 1.0, 2.5, 5.0, 10.0,
        ));
        Buckets {
            default: vec!(
                10f64, 30f64, 100f64, 300f64, 1000f64, 3000f64, 10000f64, 30000f64, 100000f64,
            ),
            custom_buckets,
        }
    }
}
