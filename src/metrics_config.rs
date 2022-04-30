use serde::Deserialize;
use std::time::Duration;

/// Available Sink types.
///
/// Influx will generate metrics in InfluxDB
/// [line](https://docs.influxdata.com/influxdb/v1.7/write_protocols/line_protocol_reference/#line-protocol-syntax)
/// format.
///
/// Graphite will generate metrics in Graphite
/// [plaintext](https://graphite.readthedocs.io/en/latest/feeding-carbon.html#the-plaintext-protocol)
/// format.
#[derive(Debug, Deserialize, Clone)]
pub enum SinkType {
    Influx,
    Graphite,
}

/// Config to initialize a Metrics Exporter.
#[derive(Debug, Deserialize, Clone)]
pub struct MetricsConfig {
    /// The \<ip:port\> to which the metrics will be shipped
    pub endpoint: String,

    /// The SinkType that should be used to receive and format the metrics
    pub sink_type: SinkType,

    /// A prefix that will be prepended to all metric names.
    ///
    /// E.g., with `prefix: dev_`, and the following metric emission:
    /// ```no_run
    /// # use fluxite::*;
    /// count!("my_api", 1, "user" => "foo");
    /// ```
    ///
    /// Will emit the following InfluxDB metric: `dev_my_api,user=foo count=1 <timestamp>`
    pub prefix: Option<String>,

    /// The interval for how often metrics are emitted by the exporter. Default is 5 seconds.
    pub interval: Option<Duration>,
}
