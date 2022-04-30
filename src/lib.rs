//! Metrics exportation library.
//!
//! This library is used to emit metrics in either [InfluxDB](https://www.influxdata.com/) linefeed
//! OR [Graphite](https://graphiteapp.org/) plaintext format and exporting them over UDP.
use core::sync::atomic::{AtomicUsize, Ordering};
use crossbeam_channel::{unbounded, Receiver, Sender};
use log::info;
use std::{thread, time::Duration};

// Copying how this works from `metrics-rs` library
static mut SINK: &'static dyn Sink = &NoopSink;
static STATE: AtomicUsize = AtomicUsize::new(0);
const INITIALIZED: usize = 1;

mod exporters;
mod metrics_config;
mod sinks;

use exporters::udp::UdpExporter;
pub use fluxite_macro::count;
pub use metrics_config::{MetricsConfig, SinkType};
pub use sinks::{graphite::GraphiteSink, influx::InfluxSink, sink::NoopSink, sink::Sink};

#[derive(Debug)]
#[doc(hidden)]
pub struct Label<'a>(&'a str, &'a str);

impl<'a> Label<'a> {
    pub fn from_parts(key: &'a str, val: &'a str) -> Self {
        Label(key, val)
    }
}
/// Initialize the sink and exporter with a [MetricsConfig].
///
/// Initialization should occur at application startup.
/// # Example
/// ```no_run
/// use fluxite::{MetricsConfig, SinkType, init_exporter};
///
/// let config = MetricsConfig {
///     endpoint: "localhost:8089".to_string(),
///     sink_type: SinkType::Influx,
///     interval: None,
///     prefix: None,
/// };
/// init_exporter(&config).unwrap();
/// ```
pub fn init_exporter(settings: &MetricsConfig) -> Result<(), String> {
    let prefix = settings.prefix.clone().unwrap_or("".to_string());
    let endpoint = settings.endpoint.clone();
    let interval = settings.interval.clone().unwrap_or(Duration::from_secs(5));

    let (tx, rx): (Sender<String>, Receiver<String>) = unbounded();
    match settings.sink_type {
        SinkType::Influx => {
            let s = InfluxSink::new(&prefix, tx);
            unsafe {
                SINK = &*Box::into_raw(Box::new(s));
                STATE.store(INITIALIZED, Ordering::SeqCst);
            }
        }
        SinkType::Graphite => {
            let s = GraphiteSink::new(&prefix, tx);
            unsafe {
                SINK = &*Box::into_raw(Box::new(s));
                STATE.store(INITIALIZED, Ordering::SeqCst);
            }
        }
    }

    thread::spawn(move || UdpExporter::new(interval, endpoint, rx).run());

    info!("Successfully setup metrics");

    Ok(())
}

#[doc(hidden)]
pub fn get_sink() -> Option<&'static dyn Sink> {
    if STATE.load(Ordering::Relaxed) != INITIALIZED {
        return None;
    }

    unsafe { Some(SINK) }
}
