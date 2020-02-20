use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub enum ExporterType {
    UDP,
}

#[derive(Debug, Deserialize, Clone)]
pub enum ObserverType {
    Influx,
}

/// Config to initialize a Metrics Reporter.
#[derive(Debug, Deserialize, Clone)]
pub struct MetricsConfig {
    pub exporter_type: ExporterType,
    pub endpoint: String,
    pub observer_type: ObserverType,
    pub prefix: Option<String>,
}
