#![allow(dead_code)]
#![allow(unused_imports)]
use log::{error, info};
use gethostname::gethostname;
use metrics::Recorder;
use metrics_core::Key;
use std::net::UdpSocket;
use std::time::SystemTime;
pub use metrics;

mod metrics_config;
pub type MetricsConfig = metrics_config::MetricsConfig;

trait RecorderBase {
    fn format_metrics(&self, key: Key, value: u64) -> String;
    fn send(&self, sock: &UdpSocket, endpoint: &str, data: String) {
        if let Err(e) = sock.send_to(&data.into_bytes(), endpoint) {
            println!("{:?}", e);
        }
    }
}

struct InfluxRecorder {
    prefix: String,
}

impl InfluxRecorder {
    fn new(cfg: &MetricsConfig) -> Self {
        let hostname = gethostname().into_string().unwrap();
        let prefix = format!("{},cluster={},host={}", cfg.app_name, cfg.cluster_name, hostname);
        Self{prefix}
    }
}
struct GraphiteRecorder {
    prefix: String,
}

impl GraphiteRecorder {
    fn new(cfg: &MetricsConfig) -> Self {
        let hostname = gethostname().into_string().unwrap();
        let prefix = format!("{}.{}.{}", cfg.app_name, cfg.cluster_name, hostname);
        Self {prefix}
    }
}
struct BlackholeRecorder {
    prefix: String,
}

impl BlackholeRecorder {
    fn new(cfg: &MetricsConfig) -> Self {
        let prefix = format!("cluster={}, app={}", cfg.cluster_name, cfg.app_name);
        Self{prefix}
    }
}

struct MetricsRecorder {
    endpoint: String,
    sock: UdpSocket,
    recorder: Box<dyn RecorderBase>,
}

impl RecorderBase for InfluxRecorder {
    fn format_metrics(&self, key: Key, value: u64) -> String {
        let (name, labels) = key.into_parts();
        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
        if labels.is_empty() {
                format!("{} {}={} {}", self.prefix, name, value, now.as_nanos())
        } else {
            let kv_pairs = labels
                .iter()
                .map(|label| format!("{}={}", label.key(), label.value()))
                .collect::<Vec<_>>();
            format!("{},{} {}={} {}", self.prefix, kv_pairs.join(","), name, value, now.as_nanos())
        }
    }
}

impl RecorderBase for GraphiteRecorder {
    fn format_metrics(&self, key: Key, value: u64) -> String {
        let (name, _labels) = key.into_parts();
        format!("{}.{} {}", self.prefix, name, value)
    }
}

impl RecorderBase for BlackholeRecorder {
    fn format_metrics(&self, key: Key, value: u64) -> String {
        let (name, _labels) = key.into_parts();
        format!("[{:?}] {}: {}={}", SystemTime::now(), self.prefix, name, value)
    }

    fn send(&self, _: &UdpSocket, _: &str, data: String) {
        println!("{}", data);
    }
}


fn build_reporter(cfg: &MetricsConfig) -> Box<dyn RecorderBase> {
    match cfg.metrics_type.as_ref() {
        "influx" => Box::new(InfluxRecorder::new(&cfg)),
        "graphite" => Box::new(GraphiteRecorder::new(&cfg)),
        _ => Box::new(BlackholeRecorder::new(&cfg)),
    }
}

impl MetricsRecorder {

    fn new(cfg: &MetricsConfig) -> Self {
        let sock = UdpSocket::bind("0.0.0.0:0").expect("failed to bind host socket");
        let recorder = build_reporter(&cfg);
        Self{endpoint: cfg.endpoint.to_string(), sock, recorder}
    }

    fn send(&self, key: Key, value: u64) {
        let msg = self.recorder.format_metrics(key, value);
        self.recorder.send(&self.sock, &self.endpoint, msg)
    }
}


impl Recorder for MetricsRecorder {
    fn increment_counter(&self, key: Key, value: u64) {
        self.send(key, value);
    }

    fn update_gauge(&self, key: Key, value: i64) {
        self.send(key, value as u64);
    }

    fn record_histogram(&self, key: Key, value: u64) {
        self.send(key, value);
    }
}

pub fn init_reporter(settings: &MetricsConfig) -> Result<(), String> {
    let recorder = MetricsRecorder::new(settings);
    
    match metrics::set_boxed_recorder(Box::new(recorder)) {
        Ok(r) => {
            info!("Successfully setup MetricsRecorder: {:?}", r);
            Ok(())
        }
        Err(e) => {
            let err_msg = format!("Error on init: {:?}", e);
            error!("{}", err_msg);
            Err(err_msg)
        }
    }
}
