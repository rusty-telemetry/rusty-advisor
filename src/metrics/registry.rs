use std::borrow::Borrow;
use std::sync::Arc;

use dashmap::DashMap;
use dashmap::mapref::entry::Entry;
use tokio::sync::RwLock;

use crate::errors::Error::MetricAlreadyRegDifferently;
use crate::errors::Result;
use crate::metrics::histogram::{Histogram, HistogramBuilder, HistogramRecorder};
use crate::metrics::metric::{MetricDescription, MetricId, MetricName};

lazy_static! {
    pub static ref GLOBAL_REGISTRY: Registry = Registry::new("GlobalMetricRegistry".to_string());
}

pub fn global_registry() -> &'static Registry {
    GLOBAL_REGISTRY.borrow()
}

pub(crate) type MetricsStorage<T> = DashMap<MetricName, MetricHolder<T>>;

pub struct Registry {
    name: String,
    histograms_storage: MetricsStorage<Histogram>,
}

impl Registry {
    pub fn new(name: String) -> Registry {
        Registry {
            name,
            histograms_storage: DashMap::default(),
        }
    }

    pub async fn get_or_register_histogram(&self, histogram_builder: HistogramBuilder) -> Result<HistogramRecorder> {
        let metric_desc = histogram_builder.metric_description()?;
        debug!("Adding histogram {} on Registry {}. [Description: {}. Settings: {}. Tags: {:#?}]", histogram_builder.name,
               self.name, histogram_builder.description, histogram_builder.settings, histogram_builder.tags);
        Self::get_or_add_metric(&self.histograms_storage, metric_desc.clone(),
                                |metric_desc| {
                                    debug!("Crating histogram {}", histogram_builder.name);
                                    Histogram::new(metric_desc, histogram_builder.settings).unwrap()
                                },
                                |metric| {
                                    metric.new_recorder()
                                }).await
    }

    pub(crate) async fn get_or_add_metric<F, T, R, FR>(metrics_storage: &MetricsStorage<T>, metric_description: MetricDescription,
                                                       builder: F, new_recorder: FR) -> Result<R>
        where
            F: FnOnce(MetricDescription) -> T,
            FR: FnOnce(&T) -> R,
    {
        let name = metric_description.name.clone();
        let metric_id = metric_description.id;
        let metric_description_copy = metric_description.clone();
        let metric_holder = metrics_storage
            .entry(name)
            .or_insert_with(|| { MetricHolder::<T>::new(metric_description_copy) });
        if metric_holder.metric_description.definition_hash() != metric_description.definition_hash() {
            Result::Err(MetricAlreadyRegDifferently())
        } else {
            let entry = metric_holder.metrics.entry(metric_id);
            match entry {
                Entry::Occupied(entry) => {
                    let ref_mut = entry.into_ref();
                    let metric = ref_mut.value().read().await;
                    let recorder = new_recorder(&metric);
                    Result::Ok(recorder)
                },
                Entry::Vacant(entry) => {
                    let metric = builder(metric_description);
                    let recorder = new_recorder(&metric);
                    entry.insert(Arc::new(RwLock::new(metric)));
                    Result::Ok(recorder)
                },
            }
        }
    }

    pub fn histograms(&self) -> Vec<Arc<RwLock<Histogram>>> {
        self.histograms_storage.iter()
            .flat_map(|ref_multi| {
                ref_multi.borrow()
                    .metrics
                    .clone()
                    .iter()
                    .map(|item| { Arc::clone(item.value()) })
                    .collect::<Vec<Arc<RwLock<Histogram>>>>()
            })
            .collect::<Vec<Arc<RwLock<Histogram>>>>()
    }
}

#[derive(Clone)]
pub(crate) struct MetricHolder<T> {
    metric_description: MetricDescription,
    metrics: DashMap<MetricId, Arc<RwLock<T>>>,
}

impl<T> MetricHolder<T> {
    pub fn new(metric_definition: MetricDescription) -> MetricHolder<T> {
        MetricHolder {
            metric_description: metric_definition,
            metrics: DashMap::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::errors::Error;
    use crate::metrics::histogram::HistogramSettings;

    use super::*;

// lazy_static! {
    //     static ref defaults: Arc<Defaults> = Arc::new(Defaults::new());
    // }
    //
    // struct Defaults {
    //     metric_builder: Box<dyn FnOnce(MetricDescription) -> Histogram>,
    // }
    //
    // impl Defaults {
    //     fn new() -> Defaults {
    //         Defaults {
    //             metric_builder: Box::new(|metric_desc| {
    //                 Histogram::new(metric_desc, HistogramSettings::default()).unwrap()
    //             }),
    //         }
    //     }
    // }

    macro_rules! aw {
        ($e:expr) => {
            tokio_test::block_on($e)
        };
    }

    #[test]
    fn test_register_new_metric_properly() {
        let registry = Registry::new("GlobalMetricRegistry".into());
        let metric = MetricDescription::from("metric_name".into(), "some description".into(), hashmap! {"tag_1".into() => "tag_value_1".into()})
            .unwrap();
        let result = aw!(Registry::get_or_add_metric(&registry.histograms_storage, metric,
                                     |metric_desc| {
                                         Histogram::new(metric_desc, HistogramSettings::default()).unwrap()
                                     },
                                     |metric| {
                                         metric.new_recorder()
                                     }));
        assert!(result.is_ok(), "Result from get_or_registry should be Ok(MetricRecorder)");
        assert_eq!(result.unwrap().measurement_unit, HistogramSettings::default().measurement_unit);
    }

    #[test]
    fn test_register_metric_already_registered_with_same_values_throws_metric_already_reg() {
        let registry = Registry::new("GlobalMetricRegistry".to_string());
        let original_metric = MetricDescription::from("metric_name".to_string(), "some description".into(), hashmap! {"tag_1".into() => "tag_value_1".into()})
            .unwrap();
        let copy_of_original_metric = original_metric.clone();
        aw!(Registry::get_or_add_metric(&registry.histograms_storage, original_metric,
                                         |metric_desc| {
                                             Histogram::new(metric_desc, HistogramSettings::default()).unwrap()
                                         },
                                         |metric| {
                                             metric.new_recorder()
                                         })).expect("It should have been accepted the first attempt to register a metric");
        let result = aw!(Registry::get_or_add_metric(&registry.histograms_storage, copy_of_original_metric,
                                     |metric_desc| {
                                         Histogram::new(metric_desc, HistogramSettings::default()).unwrap()
                                     },
                                     |metric| {
                                         metric.new_recorder()
                                     }));
        assert!(result.is_ok(), "Result from get_or_registry should be Ok(MetricRecorder)");
        assert_eq!(result.unwrap().measurement_unit, HistogramSettings::default().measurement_unit);
    }

    #[test]
    fn test_register_metric_already_registered_with_different_tag_values_is_ok() {
        let registry = Registry::new("GlobalMetricRegistry".to_string());
        let original_metric = MetricDescription::from("metric_name".to_string(), "some description".to_string(), hashmap! {"tag_1".into() => "tag_value_1".into()})
            .unwrap();
        aw!(Registry::get_or_add_metric(&registry.histograms_storage, original_metric,
                                         |metric_desc| {
                                             Histogram::new(metric_desc, HistogramSettings::default()).unwrap()
                                         },
                                         |metric| {
                                             metric.new_recorder()
                                         })).expect("It should accept the first attempt to register a metric with test purposes");
        let original_metric = MetricDescription::from("metric_name".to_string(), "some description".to_string(), hashmap! {"tag_1".into() => "tag_value_2".into()})
            .unwrap();
        let result = aw!(Registry::get_or_add_metric(&registry.histograms_storage, original_metric,
                                     |metric_desc| {
                                         Histogram::new(metric_desc, HistogramSettings::default()).unwrap()
                                     },
                                     |metric| {
                                         metric.new_recorder()
                                     }));
        assert!(result.is_ok(), "Result from get_or_registry should be Ok(MetricRecorder)");
        assert_eq!(result.unwrap().measurement_unit, HistogramSettings::default().measurement_unit);
    }

    #[test]
    fn test_register_metric_already_registered_with_different_description_throws_metric_already_reg() {
        let registry = Registry::new("GlobalMetricRegistry".to_string());
        let original_metric = MetricDescription::from("metric_name".to_string(), "some description".to_string(), hashmap! {"tag_1".into() => "tag_value_1".into()})
            .unwrap();
        aw!(Registry::get_or_add_metric(&registry.histograms_storage, original_metric,
                                         |metric_desc| {
                                             Histogram::new(metric_desc, HistogramSettings::default()).unwrap()
                                         },
                                         |metric| {
                                             metric.new_recorder()
                                         })).expect("It should accept the first attempt to register a metric with test purposes");
        let original_metric = MetricDescription::from("metric_name".to_string(), "another description".to_string(), hashmap! {"tag_1".into() => "tag_value_2".into()})
            .unwrap();
        let copy_of_sent_metric = original_metric.clone();
        let result = aw!(Registry::get_or_add_metric(&registry.histograms_storage, original_metric,
                                                 |metric_desc| {
                                                     Histogram::new(metric_desc, HistogramSettings::default()).unwrap()
                                                 },
                                                 |metric| {
                                                     metric.new_recorder()
                                                 }));
        match result {
            Result::Err(Error::MetricAlreadyRegDifferently()) =>
                (), // no more assert is required
            other => panic!("Result from get_or_registry should be Error(MetricAlreadyRegDifferently).\n\nMetric sent: {:#?}\n\n Recorder received: {:#?}", copy_of_sent_metric, other)
        };
    }
}
