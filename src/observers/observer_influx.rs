use hdrhistogram::Histogram;
use metrics_core::{Builder, Drain, Key, Label, Observer};
use metrics_util::{parse_quantiles, MetricsTree, Quantile};
use std::collections::HashMap;
use std::time::SystemTime;

/// Builder for [`InfluxObserver`].
pub struct InfluxBuilder {
    quantiles: Vec<Quantile>,
}

impl InfluxBuilder {
    /// Creates a new [`InfluxBuilder`] with default values.
    pub fn new() -> Self {
        let quantiles = parse_quantiles(&[0.0, 0.5, 0.8, 0.99, 1.0]);

        Self {
            quantiles,
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
            tree: MetricsTree::default(),
            histos: HashMap::new(),
            dw_metrics: Vec::new(),
        }
    }
}

impl Default for InfluxBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Observes metrics in InfluxDB format.
pub struct InfluxObserver {
    pub quantiles: Vec<Quantile>,
    pub tree: MetricsTree,
    pub histos: HashMap<Key, Histogram<u64>>,
    pub dw_metrics: Vec<String>,
}

impl Observer for InfluxObserver {
    fn observe_counter(&mut self, key: Key, value: u64) {
        let m = format_metrics(key, value);
        self.dw_metrics.push(m);
    }

    fn observe_gauge(&mut self, key: Key, value: i64) {
        let (levels, name) = key_to_parts(key);
        self.tree.insert_value(levels, name, value);
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
        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
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
        self.tree.clear();
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

fn format_metrics(key: Key, value: u64) -> String {
    let (name, labels) = key.into_parts();
    let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    if labels.is_empty() {
            format!("{} count={} {}", name, value, now.as_nanos())
    } else {
        let kv_pairs = labels
            .iter()
            .map(|label| format!("{}={}", label.key(), label.value()))
            .collect::<Vec<_>>();
        format!("{},{} count={} {}", name, kv_pairs.join(","), value, now.as_nanos())
    }
}

fn key_to_parts(key: Key) -> (Vec<String>, String) {
    let (name, labels) = key.into_parts();
    let mut parts = name.split('.').map(ToOwned::to_owned).collect::<Vec<_>>();
    let name = parts.pop().expect("name didn't have a single part");

    let labels = labels
        .into_iter()
        .map(Label::into_parts)
        .map(|(k, v)| format!("{}=\"{}\"", k, v))
        .collect::<Vec<_>>()
        .join(",");
    let label = if labels.is_empty() {
        String::new()
    } else {
        format!("{{{}}}", labels)
    };

    let fname = format!("{}{}", name, label);

    (parts, fname)
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