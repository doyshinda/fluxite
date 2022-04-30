use super::sink::Sink;
use super::utils::epoch_time;
use crate::Label;
use crossbeam_channel::Sender;

/// A Sink that converts metrics to Graphite plaintext format.
pub struct GraphiteSink {
    prefix: String,
    tx: Sender<String>,
}

impl GraphiteSink {
    pub fn new(prefix: &str, tx: Sender<String>) -> GraphiteSink {
        GraphiteSink {
            prefix: prefix.to_string(),
            tx,
        }
    }

    fn format_metric(&self, key: &str, value: usize) -> String {
        format!(
            "{}{} {} {}",
            self.prefix,
            key,
            value,
            epoch_time().as_nanos()
        )
    }
}

impl Sink for GraphiteSink {
    fn count(&self, key: &str, val: usize) {
        self.count_with_labels(key, val, &vec![]);
    }

    fn count_with_labels(&self, key: &str, val: usize, _labels: &Vec<Label>) {
        let m = self.format_metric(key, val);
        self.tx.send(m).unwrap();
    }
}
