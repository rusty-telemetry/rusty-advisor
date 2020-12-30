//!
//! Base on Kamon units <https://kamon.io/>
//!

use std::fmt;
use std::fmt::Display;

use serde::export::Formatter;

lazy_static! {
    pub static ref MEASUREMENT_UNITS: MeasurementUnits = MeasurementUnits::new();
}

pub fn convert(value: f64, from: &MeasurementUnit, to: &MeasurementUnit) -> f64 {
    if from.dimension != to.dimension {
        warn!("Can't convert values from {:?} dimension into {:?} dimension.", from.dimension, to.dimension);
        value
    } else if from == to {
        value
    } else {
        (from.magnitude.factor / to.magnitude.factor) * value
    }
}

pub struct MeasurementUnits {
    pub time: TimeUnits,
    pub information: InformationUnits,
    pub percentage: MeasurementUnit,
    pub none: MeasurementUnit,
}

impl MeasurementUnits {
    fn new() -> MeasurementUnits {
        MeasurementUnits {
            time: TimeUnits::new(),
            information: InformationUnits::new(),
            percentage: MeasurementUnit::new(Dimension::Percentage, Magnitude::new("percentage".into(), 1 as f64)),
            none: MeasurementUnit::new(Dimension::None, Magnitude::new("none".into(), 1 as f64)),
        }
    }
}

pub struct TimeUnits {
    pub nanos: MeasurementUnit,
    pub micros: MeasurementUnit,
    pub millis: MeasurementUnit,
    pub seconds: MeasurementUnit,
}

impl TimeUnits {
    fn new() -> TimeUnits {
        TimeUnits {
            nanos: MeasurementUnit::new(Dimension::Time, Magnitude::new("nanoseconds".into(), 1e-9)),
            micros: MeasurementUnit::new(Dimension::Time, Magnitude::new("microseconds".into(), 1e-6)),
            millis: MeasurementUnit::new(Dimension::Time, Magnitude::new("milliseconds".into(), 1e-3)),
            seconds: MeasurementUnit::new(Dimension::Time, Magnitude::new("seconds".into(), 1 as f64)),
        }
    }
}

pub struct InformationUnits {
    pub bytes: MeasurementUnit,
    pub kilobytes: MeasurementUnit,
    pub megabytes: MeasurementUnit,
    pub gigabytes: MeasurementUnit,
}

impl InformationUnits {
    fn new() -> InformationUnits {
        InformationUnits {
            bytes: MeasurementUnit::new(Dimension::Information, Magnitude::new("bytes".into(), 1 as f64)),
            kilobytes: MeasurementUnit::new(Dimension::Information, Magnitude::new("kilobytes".into(), (1u64 << 10) as f64)),
            megabytes: MeasurementUnit::new(Dimension::Information, Magnitude::new("megabytes".into(), (1u64 << 20) as f64)),
            gigabytes: MeasurementUnit::new(Dimension::Information, Magnitude::new("gigabytes".into(), (1u64 << 30) as f64)),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct MeasurementUnit {
    dimension: Dimension,
    magnitude: Magnitude,
}

impl MeasurementUnit {
    pub fn new(dimension: Dimension, magnitude: Magnitude) -> MeasurementUnit {
        MeasurementUnit {
            dimension,
            magnitude,
        }
    }
}

impl Display for MeasurementUnit {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Unit{{dim: {:?}, magnitude: {} with factor {}}}", self.dimension, self.magnitude.name, self.magnitude.factor)
    }
}

#[derive(Debug, PartialEq)]
pub enum Dimension {
    None,
    Time,
    Percentage,
    Information,
}

#[derive(Debug, PartialEq)]
pub struct Magnitude {
    name: String,
    factor: f64,
}

impl Magnitude {
    fn new(name: String, factor: f64) -> Magnitude {
        Magnitude {
            name,
            factor,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_from_nanos_to_seconds() {
        let from = &MEASUREMENT_UNITS.time.nanos;
        let to = &MEASUREMENT_UNITS.time.seconds;
        let result = convert(1025 as f64, from, to);
        let expected = 0.000_001_025 as f64;
        println!("{} == {}", result, expected);
        assert!(approx_eq!(f64, result, expected, ulps = 9));
    }

    #[test]
    fn test_convert_from_seconds_to_nanos() {
        let from = &MEASUREMENT_UNITS.time.seconds;
        let to = &MEASUREMENT_UNITS.time.nanos;
        let result = convert(1025 as f64, from, to);
        let expected = 1_025_000_000_000f64;
        println!("{} == {}", result, expected);
        assert!(approx_eq!(f64, result, expected, ulps = 9));
    }
}
