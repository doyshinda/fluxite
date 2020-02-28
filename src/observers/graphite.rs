use hdrhistogram::Histogram;
use log::debug;
use metrics_core::{Builder, Drain, Key, Observer};
use metrics_util::{parse_quantiles, Quantile};
use std::collections::HashMap;

use super::utils::{epoch_time, hist_to_values};

/// Builder for [GraphiteObserver](GraphiteObserver).
pub struct GraphiteBuilder {
    quantiles: Vec<Quantile>,
    prefix: String,
}

impl GraphiteBuilder {
    /// Creates a new [GraphiteBuilder](GraphiteBuilder) with default values.
    ///
    /// See [GraphiteObserver](GraphiteObserver) for usage of `prefix`.
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

impl Builder for GraphiteBuilder {
    type Output = GraphiteObserver;

    fn build(&self) -> Self::Output {
        let prefix = match &self.prefix.len() {
            0 => "".to_string(),
            _ => format!("{}.", self.prefix),
        };

        GraphiteObserver {
            quantiles: self.quantiles.clone(),
            histos: HashMap::new(),
            metrics: Vec::new(),
            prefix,
        }
    }
}

/// Observes metrics in Graphite plaintext format.
pub struct GraphiteObserver {
    quantiles: Vec<Quantile>,
    histos: HashMap<Key, Histogram<u64>>,
    metrics: Vec<String>,

    /// A string that will be prepended to every metric sent to Graphite.
    ///
    /// E.g., with `prefix="my_cluster.my_app"`, generating a metric like this:
    /// ```
    /// counter!("my_count", 1);
    /// ```
    /// will send the following to Graphite:
    /// ```
    /// my_cluster.my_app.my_count 1
    /// ```
    pub prefix: String,
}

impl GraphiteObserver {
    fn format_metrics(&self, key: Key, value: u64) -> String {
        let (name, _) = key.into_parts();
        let now = epoch_time();
        format!("{}{} {} {}", self.prefix, name, value, now.as_nanos())
    }
}

impl Observer for GraphiteObserver {
    fn observe_counter(&mut self, key: Key, value: u64) {
        let m = self.format_metrics(key, value);
        debug!("metric: {}", m);
        self.metrics.push(m);
    }

    fn observe_gauge(&mut self, key: Key, value: i64) {
        let m = self.format_metrics(key, value as u64);
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

impl Drain<String> for GraphiteObserver {
    fn drain(&mut self) -> String {
        let now = epoch_time();
        for (key, h) in self.histos.drain() {
            let (name, _) = key.into_parts();
            let values = hist_to_values(&h, &self.quantiles, |a, b| format!("{} {}", a, b));
            for hist_value in values {
                let m = format!("{}{}.{} {}", self.prefix, name, hist_value, now.as_nanos());
                self.metrics.push(m);
            }
        }

        let rendered = self.metrics.join("\n");
        self.metrics.clear();
        rendered
    }
}
