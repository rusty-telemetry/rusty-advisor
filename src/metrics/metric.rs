//! For metric definition we adopt the same notation as Prometheus and OpenTSDB
//!
//! Details of required format can be found at
//!   - https://prometheus.io/docs/concepts/data_model/#metric-names-and-labels
//!   - http://opentsdb.net/docs/build/html/user_guide/writing/index.html

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use crate::errors::{Error, Result};

pub type MetricId = u64;
pub type MetricDefinitionHash = u64;
pub type MetricName = String;

/// Holds the definition for a concrete metric.
#[derive(Clone, Debug, PartialEq)]
pub struct MetricDescription {
    /// `id` is a hash of the values of the name and the tag values.
    /// It can be used as identifier of a metric among all metrics.
    pub id: MetricId,
    /// `definition_hash` is a hash of the values of the name, the description and the tag names.
    /// It can be used as identifier of the dimension definition.
    /// A dimension can contain multiple concrete metrics, each one with different tag values.
    pub definition_hash: MetricDefinitionHash,
    pub name: MetricName,
    pub description: String,
    pub tag_names: Vec<String>,
    pub tags: HashMap<String, String>,
}

#[derive(Debug)]
pub enum MetricKind {
    Counter,
    Gauge,
    Histogram,
}

impl MetricDescription {
    pub fn from(name: String, description: String, tags: HashMap<String, String>) -> Result<MetricDescription> {
        Self::validate_name(&name)
            .and_then(|_| {
                Self::validate_tag_values(&tags)
            })
            .and_then(|_| {
                let mut tag_names: Vec<String> = Vec::with_capacity(tags.len());
                for (tag_name, _) in tags.iter() {
                    tag_names.push(tag_name.clone());
                }
                Self::validate_tag_names(&tag_names)
                    .map(|_| tag_names)
            })
            .map(|tag_names| {
                let id: u64 = Self::compute_metric_id(&name, &tags);
                let definition_hash: u64 = Self::compute_metric_definition_hash(&name, &description, &tag_names);
                MetricDescription {
                    id,
                    definition_hash,
                    name,
                    description,
                    tag_names,
                    tags,
                }
            })
    }

    // FIXME: improve with a hash function over each value wich ignore order (like Arrays#hash() in Java)
    fn compute_metric_id(name: &String, tags: &HashMap<String, String>) -> u64 {
        let mut values: Vec<&str> = Vec::with_capacity(tags.len() + 1);
        values.push(name);
        for (_, tag_value) in tags.iter() {
            values.push(tag_value);
        }
        values.sort();
        let mut hash = DefaultHasher::new();
        values.hash(&mut hash);
        hash.finish()
    }

    // FIXME: improve with a hash function over each value which ignore order (like Arrays#hash() in Java)
    fn compute_metric_definition_hash(name: &String, description: &String, tags: &Vec<String>) -> u64 {
        let mut values: Vec<&str> = Vec::with_capacity(tags.len() + 2);
        values.push(name);
        values.push(description);
        for tag_name in tags.iter() {
            values.push(tag_name);
        }
        values.sort();
        let mut hash = DefaultHasher::new();
        values.hash(&mut hash);
        hash.finish()
    }

    fn validate_name(name: &String) -> Result<()> {
        if !is_tag_metric_name(&name) {
            return Err(Error::Msg(format!("'{}' is not a valid metric name. It must match regex [a-zA-Z_:][a-zA-Z0-9_:]*", name)));
        }
        Ok(())
    }

    fn validate_tag_names(tag_names: &Vec<String>) -> Result<()> {
        for tag_name in tag_names.iter() {
            if !is_valid_tag_name(&tag_name) {
                return Err(Error::Msg(format!("'{}' is not a valid tag name. It must match regex [a-zA-Z_][a-zA-Z0-9_]*", tag_name)));
            }
        }
        Ok(())
    }

    fn validate_tag_values(tags: &HashMap<String, String>) -> Result<()> {
        for tag_value in tags.values() {
            if !is_valid_tag_value(&tag_value) {
                return Err(Error::Msg(format!("'{}' is not a valid tag value. It must match regex [a-zA-Z0-9-_./]*", tag_value)));
            }
        }
        Ok(())
    }

    pub fn definition_hash(&self) -> MetricDefinitionHash {
        self.definition_hash
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn tags(&self) -> &HashMap<String, String> {
        &self.tags
    }
}

/// Valid metric names must match regex [a-zA-Z_:][a-zA-Z0-9_:]*.
fn is_tag_metric_name(name: &str) -> bool {
    fn valid_start(c: char) -> bool {
        c.is_ascii()
            && match c as u8 {
            b'a'..=b'z' | b'A'..=b'Z' | b'_' | b':' => true,
            _ => false,
        }
    }

    fn valid_char(c: char) -> bool {
        c.is_ascii()
            && match c as u8 {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b':' => true,
            _ => false,
        }
    }

    name.starts_with(valid_start) && !name.contains(|c| !valid_char(c))
}

/// Valid tag names must match regex [a-zA-Z_][a-zA-Z0-9_]*.
fn is_valid_tag_name(name: &str) -> bool {
    fn valid_start(c: char) -> bool {
        c.is_ascii()
            && match c as u8 {
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => true,
            _ => false,
        }
    }

    fn valid_char(c: char) -> bool {
        c.is_ascii()
            && match c as u8 {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' => true,
            _ => false,
        }
    }

    name.starts_with(valid_start) && !name.contains(|c| !valid_char(c))
}

/// Valid tag values must match regex [a-zA-Z0-9-_./]*.
///
/// More details can be found at
///   http://opentsdb.net/docs/build/html/user_guide/writing/index.html#metrics-and-tags
fn is_valid_tag_value(name: &str) -> bool {
    fn valid_char(c: char) -> bool {
        c.is_ascii()
            && match c as u8 {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'/' => true,
            _ => false,
        }
    }

    !name.contains(|c| !valid_char(c))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_def_id_when_same_names_and_same_tag_names() {
        let metric_1 = MetricDescription::from("metric_name".into(), "some description".into(), hashmap! {"tag_1".into() => "tag_value_1".into(), "tag_2".into() => "tag_value_2".into()}).unwrap();
        let metric_2 = MetricDescription::from("metric_name".into(), "some description".into(), hashmap! {"tag_1".into() => "tag_value_1".into(), "tag_2".into() => "tag_value_2".into()}).unwrap();

        assert_eq!(metric_1.definition_hash(), metric_2.definition_hash());
    }

    #[test]
    fn diff_def_id_when_diff_name() {
        let metric_1 = MetricDescription::from("metric_name_1".into(), "some description".into(), hashmap! {"tag_1".into() => "tag_value_1".into(), "tag_2".into() => "tag_value_2".into()}).unwrap();
        let metric_2 = MetricDescription::from("metric_name_2".into(), "some description".into(), hashmap! {"tag_1".into() => "tag_value_1".into(), "tag_2".into() => "tag_value_2".into()}).unwrap();

        assert_ne!(metric_1.definition_hash(), metric_2.definition_hash());
    }

    #[test]
    fn diff_def_id_when_same_name_and_diff_description() {
        let metric_1 = MetricDescription::from("metric_name".into(), "some description".into(), hashmap! {"tag_1".into() => "tag_value_1".into(), "tag_2".into() => "tag_value_2".into()}).unwrap();
        let metric_2 = MetricDescription::from("metric_name".into(), "another description".into(), hashmap! {"tag_1".into() => "tag_value_1".into(), "tag_2".into() => "tag_value_2".into()}).unwrap();

        assert_ne!(metric_1.definition_hash(), metric_2.definition_hash());
    }

    #[test]
    fn diff_def_id_when_diff_tag_names() {
        let metric_1 = MetricDescription::from("metric_name_1".into(), "some description".into(), hashmap! {"tag_1".into() => "tag_value_1".into(), "tag_2".into() => "tag_value_2".into()}).unwrap();
        let metric_2 = MetricDescription::from("metric_name_1".into(), "some description".into(), hashmap! {"tag_1".into() => "tag_value_1".into(), "tag_3".into() => "tag_value_2".into()}).unwrap();

        assert_ne!(metric_1.definition_hash(), metric_2.definition_hash());
    }

    #[test]
    fn same_def_id_ignoring_order() {
        let metric_1 = MetricDescription::from("metric_name_1".into(), "some description".into(), hashmap! {"tag_1".into() => "tag_value_1".into(), "tag_2".into() => "tag_value_2".into()}).unwrap();
        let metric_2 = MetricDescription::from("metric_name_1".into(), "some description".into(), hashmap! {"tag_2".into() => "tag_value_2".into(), "tag_1".into() => "tag_value_1".into()}).unwrap();

        assert_eq!(metric_1.definition_hash(), metric_2.definition_hash());
    }

    #[test]
    fn same_metric_id_when_same_names_and_same_tags() {
        let metric_1 = MetricDescription::from("metric_name_1".into(), "some description".into(), hashmap! {"tag_1".into() => "tag_value_1".into(), "tag_2".into() => "tag_value_2".into()}).unwrap();
        let metric_2 = MetricDescription::from("metric_name_1".into(), "some description".into(), hashmap! {"tag_1".into() => "tag_value_1".into(), "tag_2".into() => "tag_value_2".into()}).unwrap();

        assert_eq!(metric_1.definition_hash(), metric_2.definition_hash());
        assert_eq!(metric_1.id, metric_2.id);
    }

    #[test]
    fn same_metric_id_ignoring_order() {
        let metric_1 = MetricDescription::from("metric_name_1".into(), "some description".into(), hashmap! {"tag_1".into() => "tag_value_1".into(), "tag_2".into() => "tag_value_2".into()}).unwrap();
        let metric_2 = MetricDescription::from("metric_name_1".into(), "some description".into(), hashmap! {"tag_2".into() => "tag_value_2".into(), "tag_1".into() => "tag_value_1".into()}).unwrap();

        assert_eq!(metric_1.definition_hash(), metric_2.definition_hash());
        assert_eq!(metric_1.id, metric_2.id);
    }

    #[test]
    fn same_def_id_and_diff_metric_id_when_same_names_and_diff_tag_valuss() {
        let metric_1 = MetricDescription::from("metric_name_1".into(), "some description".into(), hashmap! {"tag_1".into() => "tag_value_1".into(), "tag_2".into() => "tag_value_2".into()}).unwrap();
        let metric_2 = MetricDescription::from("metric_name_1".into(), "some description".into(), hashmap! {"tag_1".into() => "tag_value_1".into(), "tag_2".into() => "tag_value_3".into()}).unwrap();

        assert_eq!(metric_1.definition_hash(), metric_2.definition_hash());
        assert_ne!(metric_1.id, metric_2.id);
    }
}
