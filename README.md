# fluxite

This library is a thin wrapper around [metrics-runtime](https://docs.rs/metrics-runtime/0.13.0/metrics_runtime/index.html) that supports formatting metrics in [InfluxDB](https://www.influxdata.com/) linefeed and [Graphite](https://graphiteapp.org/) plaintext format and exporting them over UDP.

## Example
The reporter should be initialized once at application startup:
```Rust
let config = MetricsConfig {
    endpoint: "localhost:8089",
    observer_type: ObserverType::Influx,
};

init_reporter(&config).unwrap();
```
Reporting metrics works through the use of the [metrics facade crate](https://docs.rs/metrics/0.12.1/metrics/#use):
```Rust
use metrics::timing;

let start = Instant::now();
// ...
// do work
// ...
let end = Instant::now();

timing!("work_time_ns", start, end);
```

Will emit a metric with the percentiles like so:
```
work_time_ns min=100,p50=150,p75=160,p99=180,max=200 <ts>
```
