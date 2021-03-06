use hdrhistogram::Histogram;
use log::debug;
use metrics_core::{Builder, Drain, Key, Observer};
use metrics_util::{parse_quantiles, Quantile};
use std::collections::HashMap;

use super::utils::{epoch_time, hist_to_values};

/// Builder for InfluxObserver.
pub struct InfluxBuilder {
    quantiles: Vec<Quantile>,
    prefix: String,
}

impl InfluxBuilder {
    /// Creates a new [InfluxBuilder](InfluxBuilder) with default values.
    ///
    /// A CSV string of `key=value` tags that should be prepended to every metric sent to Influx.
    ///
    /// E.g., with `prefix="app=my_app,host=bar"`, generating a metric like this:
    /// ```
    /// counter!("my_count", 1);
    /// ```
    /// will send the following to InfluxDB:
    /// ```
    /// mycount,app=my_app,host=bar value=1
    /// ```
    pub fn new(prefix: Option<String>) -> Self {
        let quantiles = parse_quantiles(&[0.0, 0.5, 0.75, 0.99, 1.0]);

        Self {
            quantiles,
            prefix: prefix.unwrap_or("".to_string()),
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
        let prefix = match self.prefix.len() {
            0 => "".to_string(),
            _ => format!(",{}", self.prefix),
        };
        InfluxObserver {
            quantiles: self.quantiles.clone(),
            histos: HashMap::new(),
            metrics: Vec::new(),
            prefix,
        }
    }
}

/// Observes metrics in InfluxDB format.
pub struct InfluxObserver {
    quantiles: Vec<Quantile>,
    histos: HashMap<Key, Histogram<u64>>,
    metrics: Vec<String>,
    prefix: String,
}

impl InfluxObserver {
    fn format_metrics(&self, key: Key, value: u64, value_key: &str) -> String {
        let (name, labels) = key.into_parts();
        let now = epoch_time();

        if labels.is_empty() {
            format!(
                "{}{} {}={} {}",
                name,
                self.prefix,
                value_key,
                value,
                now.as_nanos()
            )
        } else {
            let kv_pairs = labels
                .iter()
                .map(|label| format!("{}={}", label.key(), label.value()))
                .collect::<Vec<_>>();
            format!(
                "{}{},{} {}={} {}",
                name,
                self.prefix,
                kv_pairs.join(","),
                value_key,
                value,
                now.as_nanos()
            )
        }
    }
}

impl Observer for InfluxObserver {
    fn observe_counter(&mut self, key: Key, value: u64) {
        let m = self.format_metrics(key, value, "count");
        debug!("metric: {}", m);
        self.metrics.push(m);
    }

    fn observe_gauge(&mut self, key: Key, value: i64) {
        let m = self.format_metrics(key, value as u64, "gauge");
        debug!("metric: {}", m);
        self.metrics.push(m);
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
            let values =
                hist_to_values(&h, &self.quantiles, |a, b| format!("{}={}", a, b)).join(",");
            let m = if labels.is_empty() {
                format!("{}{} {} {}", name, self.prefix, values, now.as_nanos())
            } else {
                format!(
                    "{}{},{} {} {}",
                    name,
                    self.prefix,
                    labels,
                    values,
                    now.as_nanos()
                )
            };
            self.metrics.push(m);
        }

        let rendered = self.metrics.join("\n");
        self.metrics.clear();
        rendered
    }
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
