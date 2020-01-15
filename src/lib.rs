#![allow(dead_code)]
#![allow(unused_imports)]
use log::{error, info};
use gethostname::gethostname;
use metrics_runtime::Receiver;
use metrics_core::Key;
use std::{
    net::UdpSocket,
    time::{
        Duration,
        SystemTime,
    },
    thread,
};

pub use metrics;

mod metrics_config;
mod observers;
mod exporters;

use observers::observer_influx::InfluxBuilder;
use exporters::exporter_udp::UdpExporter;
pub type MetricsConfig = metrics_config::MetricsConfig;


pub fn init_reporter(settings: &MetricsConfig) -> Result<(), String> {
    let receiver = Receiver::builder()
        .histogram(Duration::from_secs(5), Duration::from_secs(1))
        .build()
        .expect("failed to build receiver");

    let controller = receiver.controller();
    let builder = InfluxBuilder::new();
    let mut exporter = UdpExporter::new(
        controller.clone(),
        builder,
        Duration::from_secs(2),
        &settings.endpoint
    );

    thread::spawn(move || exporter.run());

    receiver.install();
    info!("Successfully setup metrics");
    
    Ok(())
}
