use hdrhistogram::Histogram;
use metrics_util::Quantile;
use std::time::{Duration, SystemTime};

pub fn epoch_time() -> Duration {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
}

pub fn hist_to_values(
    hist: &Histogram<u64>,
    quantiles: &[Quantile],
    f: fn(&str, u64) -> String,
) -> Vec<String> {
    let mut values = Vec::new();
    for quantile in quantiles {
        let value = hist.value_at_quantile(quantile.value());
        values.push(f(quantile.label(), value));
    }

    values
}
