use serde::Deserialize;
use config::{ConfigError, Config, File};

#[derive(Debug, Deserialize, Clone)]
pub struct MetricsConfig {
    pub endpoint: String,
    pub cluster_name: String,
    pub app_name: String,
    pub metrics_type: String,
}
