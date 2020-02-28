//! Metrics collection, aggregation and exportation library.
//!
//! This library is a thin wrapper around
//! [metrics-runtime](https://docs.rs/metrics-runtime/0.13.0/metrics_runtime/index.html)
//! that supports formatting metrics in InfluxDB linefeed format and exporting them over UDP.
use log::info;
use metrics_runtime::Receiver;
use std::{thread, time::Duration};

pub use metrics;

mod exporters;
mod metrics_config;
mod observers;

pub use exporters::udp::UdpExporter;
pub use metrics_config::{MetricsConfig, ObserverType};
pub use observers::{graphite::GraphiteBuilder, influx::InfluxBuilder};

/// Initialize a metrics reporter with a [MetricsConfig](metric_config::MetricConfig).
///
/// The reporter should be initialized at application startup.
/// # Example
/// ```
/// let config = MetricsConfig {
///     exporter_type: ExporterType::UDP,
///     endpoint: "localhost:8089",
///     observer_type: ObserverType::Influx,
/// };
/// init_reporter(&config).unwrap();
/// ```
pub fn init_reporter(settings: &MetricsConfig) -> Result<(), String> {
    let receiver = Receiver::builder()
        .histogram(Duration::from_secs(15), Duration::from_secs(2))
        .build()
        .expect("failed to build receiver");

    let controller = receiver.controller();
    let prefix = settings.prefix.clone();
    let endpoint = settings.endpoint.clone();
    match settings.observer_type {
        ObserverType::Influx => thread::spawn(|| {
            UdpExporter::new(
                controller,
                InfluxBuilder::new(prefix),
                Duration::from_secs(2),
                endpoint,
            )
            .run()
        }),
        ObserverType::Graphite => thread::spawn(|| {
            UdpExporter::new(
                controller,
                GraphiteBuilder::new(prefix),
                Duration::from_secs(2),
                endpoint,
            )
            .run()
        }),
    };

    receiver.install();
    info!("Successfully setup metrics");

    Ok(())
}
