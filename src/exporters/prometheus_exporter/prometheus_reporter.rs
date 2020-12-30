use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Instant;

use hyper::{
    Body,
    header::CONTENT_TYPE,
    Request, Response, Server, service::{make_service_fn, service_fn},
};
use prometheus::{Counter, Encoder, Gauge, HistogramVec, TextEncoder};
use tokio::sync::broadcast::Receiver;
use tokio::sync::RwLock;

use crate::exporters::metrics_exporter::{MetricSample, MetricsSnapshot};
use crate::exporters::prometheus_exporter::metrics::prometheus_counter::PrometheusCounter;
use crate::exporters::prometheus_exporter::metrics::prometheus_histogram::PrometheusHistogram;
use crate::exporters::prometheus_exporter::prometheus_encoder;
use crate::exporters::prometheus_exporter::prometheus_settings::PrometheusSettings;
use crate::metrics::histogram::{HistogramBuilder, HistogramRecorder, HistogramSettings};
use crate::metrics::measurement_unit::MEASUREMENT_UNITS;

lazy_static! {
    static ref HTTP_COUNTER: Counter = register_counter!(opts!(
        "prometheus_http_requests_total",
        "Total number of HTTP requests made on the Prometheus service.",
        labels! {"handler" => "all",}
    ))
    .unwrap();

    static ref HTTP_BODY_GAUGE: Gauge = register_gauge!(opts!(
        "prometheus_http_response_size_bytes",
        "The HTTP response sizes in bytes on the Prometheus service.",
        labels! {"handler" => "all",}
    ))
    .unwrap();
    static ref HTTP_REQ_HISTOGRAM: HistogramVec = register_histogram_vec!(
        "old_prometheus_http_request_duration_seconds",
        "The HTTP request latencies in seconds on the Prometheus service.",
        &["handler"]
    )
    .unwrap();
}

async fn serve_req(metrics_holder: MetricsHolder, _req: Request<Body>,
                   http_req_histo: Arc<RwLock<HistogramRecorder>>) -> Result<Response<Body>, hyper::Error> {
    let encoder = TextEncoder::new();

    HTTP_COUNTER.inc();
    let start = Instant::now();
    let mut http_req_histo_writer = http_req_histo.write().await;
    let mut rusty_timer = http_req_histo_writer.start_timer();
    let timer = HTTP_REQ_HISTOGRAM.with_label_values(&["all"]).start_timer();

    let metric_families = prometheus::gather();
    let mut buffer = vec![];

    let guard = metrics_holder.histograms.read().await;
    for histogram in guard.values() {
        prometheus_encoder::encode_histogram(histogram, &mut buffer).unwrap()
    }
    drop(guard);

    let guard = metrics_holder.counters.read().await;
    for _counter in guard.values() {
        unimplemented!()
    }
    drop(guard);

    encoder.encode(&metric_families, &mut buffer).unwrap();
    HTTP_BODY_GAUGE.set(buffer.len() as f64);

    let response = Response::builder()
        .status(200)
        .header(CONTENT_TYPE, encoder.format_type())
        .body(Body::from(buffer))
        .unwrap();

    timer.observe_duration();
    rusty_timer.close();
    let delta = start.elapsed().as_millis() as u64;
    // let mut guard = http_req_histo.write().await;
    // guard.record(delta);
    debug!("Request to /metrics took {} millis", delta);

    Ok(response)
}

#[derive(Debug, Clone)]
struct MetricsHolder {
    histograms: Arc<RwLock<HashMap<u64, PrometheusHistogram>>>,
    counters: Arc<RwLock<HashMap<u64, PrometheusCounter>>>,
}

impl Default for MetricsHolder {
    fn default() -> Self {
        MetricsHolder {
            histograms: Arc::new(RwLock::default()),
            counters: Arc::new(RwLock::default()),
        }
    }
}

pub struct PrometheusExporter {
    config: PrometheusSettings,
    handle: Option<thread::JoinHandle<()>>,
    running: Arc<AtomicBool>,
    metrics_holder: MetricsHolder,
}

impl PrometheusExporter {
    pub fn new(config: PrometheusSettings) -> PrometheusExporter {
        PrometheusExporter {
            config,
            handle: Option::None,
            running: Arc::new(AtomicBool::new(false)),
            metrics_holder: MetricsHolder::default(),
        }
    }

    pub async fn start_server(&self) {
        let addr = format!("{}:{}", self.config.host, self.config.port).parse::<SocketAddr>().unwrap();
        info!("Prometheus Exporter listening at http://{}", addr);

        let metrics_holder = MetricsHolder::clone(&self.metrics_holder);
        let prometheus_http_req_histogram = Arc::new(RwLock::new(HistogramBuilder::new(
            "prometheus_http_request_duration_seconds".into(),
            "The HTTP request latencies in seconds on the Prometheus service.".into())
            .with_settings(HistogramSettings::from(1, 600_000, 0, &MEASUREMENT_UNITS.time.millis))
            .build()
            .await
            .unwrap()));

        let serve_future = Server::bind(&addr)
            .serve(make_service_fn(move |_| {
                let mh = metrics_holder.clone();
                let http_req_histo = prometheus_http_req_histogram.clone();
                async move {
                    Ok::<_, hyper::Error>(service_fn(move |req| serve_req(MetricsHolder::clone(&mh), req, Arc::clone(&http_req_histo))))
                }
            }));

        if let Err(err) = serve_future.await {
            error!("Server error: {}", err);
        }
    }

    pub async fn listen_metrics(&self, mut receiver: Receiver<Arc<MetricsSnapshot>>) {
        self.running.store(true, Ordering::SeqCst);
        loop {
            let is_running = self.running.clone();
            match receiver.recv().await {
                Result::Ok(metrics_snapshot) => {
                    self.consume_snapshot(metrics_snapshot).await;
                    if !is_running.load(Ordering::SeqCst) {
                        break;
                    }
                },
                Result::Err(error) => {
                    error!("Error receiving metrics snapshot on Prometheus Exporter. Reason: {}", error);
                    break;
                },
            }
        }
    }

    async fn consume_snapshot(&self, metrics_snapshot: Arc<MetricsSnapshot>) {
        for sample in metrics_snapshot.samples() {
            info!("Prometheus Exporter received metrics snapshot {:?}", sample);
            match sample {
                MetricSample::Counter(_metric_desc, _counter_sample) => unimplemented!(),
                MetricSample::Gauge(_metric_desc, _gauge_sample) => unimplemented!(),
                MetricSample::Histogram(metric_desc, histogram_sample) => {
                    info!("Receiving Metric ID {}", metric_desc.id);
                    let mut guard = self.metrics_holder.histograms.write().await;
                    let prometheus_histogram = guard
                        .entry(metric_desc.id)
                        .or_insert_with(|| {
                            info!("Metric {} didn't find on Map", metric_desc.id);
                            PrometheusHistogram::new(Arc::new(metric_desc.clone()), self.config.clone()).into()
                        });
                    prometheus_histogram.add_snapshot(histogram_sample, metrics_snapshot.timestamp_in_millis());
                },
            }
        }
    }
}

