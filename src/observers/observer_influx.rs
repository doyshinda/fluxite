use hdrhistogram::Histogram;
use metrics_core::{Builder, Drain, Key, Label, Observer};
use metrics_util::{parse_quantiles, Quantile};
use std::collections::HashMap;
use std::time::SystemTime;
use log::debug;

/// Builder for [`InfluxObserver`].
pub struct InfluxBuilder {
    quantiles: Vec<Quantile>,
    app_name: String,
    cluster_name: String,
}

impl InfluxBuilder {
    /// Creates a new [`InfluxBuilder`] with default values.
    pub fn new(app_name: String, cluster_name: String) -> Self {
        let quantiles = parse_quantiles(&[0.0, 0.5, 0.75, 0.99, 1.0]);

        Self {
            quantiles,
            app_name,
            cluster_name,
        }
    }

    /// Sets the quantiles to use when rendering histograms.
    ///
    /// Quantiles represent a scale of 0 to 1, where percentiles represent a scale of 1 to 100, so
    /// a quantile of 0.99 is the 99th percentile, and a quantile of 0.99 is the 99.9th percentile.
    ///
    /// By default, the quantiles will be set to: 0.0, 0.5, 0.9, 0.95, 0.99, 0.999, and 1.0.
    pub fn set_quantiles(mut self, quantiles: &[f64]) -> Self {
        self.quantiles = parse_quantiles(quantiles);
        self
    }
}

impl Builder for InfluxBuilder {
    type Output = InfluxObserver;

    fn build(&self) -> Self::Output {
        InfluxObserver {
            quantiles: self.quantiles.clone(),
            histos: HashMap::new(),
            dw_metrics: Vec::new(),
            app_name: self.app_name.clone(),
            cluster_name: self.cluster_name.clone(),
        }
    }
}

/// Observes metrics in InfluxDB format.
pub struct InfluxObserver {
    pub quantiles: Vec<Quantile>,
    pub histos: HashMap<Key, Histogram<u64>>,
    pub dw_metrics: Vec<String>,
    app_name: String,
    cluster_name: String,
}

impl InfluxObserver {
    fn format_metrics(&self, key: Key, value: u64, value_key: &str) -> String {
        let (name, labels) = key.into_parts();
        let now = epoch_time();
        if labels.is_empty() {
                format!("{},app={},cluster={} {}={} {}", name, self.app_name, self.cluster_name,
                    value_key, value, now.as_nanos()
                )
        } else {
            let kv_pairs = labels
                .iter()
                .map(|label| format!("{}={}", label.key(), label.value()))
                .collect::<Vec<_>>();
            format!(
                "{},app={},cluster={},{} {}={} {}", name, self.app_name, self.cluster_name,
                kv_pairs.join(","), value_key, value, now.as_nanos()
            )
        }
    }
}

impl Observer for InfluxObserver {
    fn observe_counter(&mut self, key: Key, value: u64) {
        let m = self.format_metrics(key, value, "count");
        debug!("metric: {}", m);
        self.dw_metrics.push(m);
    }

    fn observe_gauge(&mut self, key: Key, value: i64) {
        let m = self.format_metrics(key, value as u64, "gauge");
        debug!("metric: {}", m);
        self.dw_metrics.push(m);
    }

    fn observe_histogram(&mut self, key: Key, values: &[u64]) {
        let entry = self
            .histos
            .entry(key)
            .or_insert_with(|| Histogram::<u64>::new(3).expect("failed to create histogram"));

        for value in values {
            entry
                .record(*value)
                .expect("failed to observe histogram value");
        }
    }
}

impl Drain<String> for InfluxObserver {
    fn drain(&mut self) -> String {
        let now = epoch_time();
        for (key, h) in self.histos.drain() {
            let (labels, name) = format_labels(key);
            let values = hist_to_values(h.clone(), &self.quantiles);
            let m = if labels.is_empty() {
                format!("{} {} {}", name, values, now.as_nanos())
            } else {
                format!("{},{} {} {}", name, labels, values, now.as_nanos())
            };
            self.dw_metrics.push(m);
        }

        let rendered = self.dw_metrics.join("\n");
        self.dw_metrics.clear();
        rendered
    }
}

fn epoch_time() -> std::time::Duration {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap()
}

fn format_labels(key: Key) -> (String, String) {
    let (name, labels) = key.into_parts();
    if labels.is_empty() {
        (String::new(), name.to_string())
    } else {
        let kv_pairs = labels
            .iter()
            .map(|label| format!("{}={}", label.key(), label.value()))
            .collect::<Vec<_>>();
        (kv_pairs.join(","), name.to_string())
    }
}

fn hist_to_values(
    hist: Histogram<u64>,
    quantiles: &[Quantile],
) -> String {
    let mut values = Vec::new();
    for quantile in quantiles {
        let value = hist.value_at_quantile(quantile.value());
        values.push(format!("{}={}", quantile.label(), value));
    }

    values.join(",")
}
