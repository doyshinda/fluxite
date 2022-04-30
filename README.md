# fluxite

This library is used to emit metrics in either [InfluxDB](https://www.influxdata.com/) linefeed OR [Graphite](https://graphiteapp.org/) plaintext format and exporting them over UDP.

## Example
The reporter should be initialized once at application startup:
```Rust
let config = MetricsConfig {
    endpoint: "localhost:8089",
    sink_type: SinkType::Influx,
    prefix: None,
    interval: None,
};

init_reporter(&config).unwrap();
```
Reporting metrics works through the use of the various macros:
```Rust
use fluxite::count;

count!("my_api", 1, "user" => "foo");
```

Will emit an InfluxDB metric like this:
```
my_api,user=foo count=1 <timestamp>
```
