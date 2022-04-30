use crate::Label;

pub trait Sink {
    /// Emit a counter metric
    fn count(&self, key: &str, val: usize);

    /// Emits a counter metric with labels (labels not supported by all SinkTypes)
    fn count_with_labels(&self, key: &str, val: usize, label: &Vec<Label>);
}

/// The default, no-op sink.
pub struct NoopSink;

impl Sink for NoopSink {
    fn count(&self, _key: &str, _val: usize) {}
    fn count_with_labels(&self, _key: &str, _val: usize, _label: &Vec<Label>) {}
}
