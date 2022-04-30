use super::sink::Sink;
use super::utils::epoch_time;
use crate::Label;
use crossbeam_channel::Sender;

/// A sink that converts metrics to InfluxDB format.
///
/// # Example
/// ```no_run
/// use fluxite::count;
///
/// count!("my_api", 1, "user" => "foo");
/// ```
///
/// Will emit a metric like this to InfluxDB: `"my_api,user=foo count=4 <timestamp>"`
pub struct InfluxSink {
    prefix: String,
    tx: Sender<String>,
}

impl InfluxSink {
    pub fn new(prefix: &str, tx: Sender<String>) -> InfluxSink {
        InfluxSink {
            prefix: prefix.to_string(),
            tx,
        }
    }

    fn format_metric(
        &self,
        key: &str,
        value: usize,
        value_key: &str,
        labels: &Vec<Label>,
    ) -> String {
        let now = epoch_time();

        if labels.is_empty() {
            format!(
                "{}{} {}={} {}",
                self.prefix,
                key,
                value_key,
                value,
                now.as_nanos()
            )
        } else {
            let kv_pairs: Vec<_> = labels
                .iter()
                .map(|Label(k, v)| format!("{}={}", k, v))
                .collect();
            format!(
                "{}{},{} {}={} {}",
                self.prefix,
                key,
                kv_pairs.join(","),
                value_key,
                value,
                now.as_nanos()
            )
        }
    }
}

impl Sink for InfluxSink {
    fn count(&self, key: &str, val: usize) {
        self.count_with_labels(key, val, &vec![]);
    }

    fn count_with_labels(&self, key: &str, val: usize, labels: &Vec<Label>) {
        let m = self.format_metric(key, val, "count", &labels);
        self.tx.send(m).unwrap();
    }
}
