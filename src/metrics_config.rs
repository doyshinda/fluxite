use serde::Deserialize;

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
