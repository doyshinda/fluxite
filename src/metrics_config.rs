use serde::Deserialize;

/// Available Observer types.
///
/// Influx will generate metrics in InfluxDB
/// [line](https://docs.influxdata.com/influxdb/v1.7/write_protocols/line_protocol_reference/#line-protocol-syntax)
/// format.
///
/// Graphite will generate metrics in Graphite
/// [plaintext](https://graphite.readthedocs.io/en/latest/feeding-carbon.html#the-plaintext-protocol)
/// format.
#[derive(Debug, Deserialize, Clone)]
pub enum ObserverType {
    Influx,
    Graphite,
}

/// Config to initialize a Metrics Reporter.
#[derive(Debug, Deserialize, Clone)]
pub struct MetricsConfig {
    pub endpoint: String,
    pub observer_type: ObserverType,
    pub prefix: Option<String>,
}
