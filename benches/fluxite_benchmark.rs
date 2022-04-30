use criterion::*;
use fluxite::{init_exporter, MetricsConfig, SinkType};
use fluxite_macro::count;

fn bench_noop_sink(c: &mut Criterion) {
    let string = String::from("blah");

    let mut group = c.benchmark_group("noop_sink");
    group.throughput(Throughput::Elements(1u64));
    group.bench_function("noop_static_metric_no_labels", |b| {
        b.iter(|| {
            count!("foo", 1);
        })
    });
    group.bench_function("noop_static_metric_static_labels", |b| {
        b.iter(|| {
            count!("foo", 1, "bar" => "blah");
        })
    });
    group.bench_function("noop_static_metric_static_key_string_val", |b| {
        b.iter(|| {
            count!("foo", 1, "bar" => string);
        })
    });
    group.finish();
}

fn bench_influx_sink(c: &mut Criterion) {
    let config = MetricsConfig {
        endpoint: "127.0.0.1:2115".to_string(),
        sink_type: SinkType::Influx,
        prefix: None,
        interval: None,
    };
    init_exporter(&config).unwrap();

    let string = String::from("blah");

    let mut group = c.benchmark_group("influx_sink");
    group.throughput(Throughput::Elements(1u64));
    group.bench_function("influx_static_metric_no_labels", |b| {
        b.iter(|| {
            count!("foo", 1);
        })
    });
    group.bench_function("influx_static_metric_static_labels", |b| {
        b.iter(|| {
            count!("foo", 1, "bar" => "blah");
        })
    });
    group.bench_function("influx_static_metric_static_key_string_val", |b| {
        b.iter(|| {
            count!("foo", 1, "bar" => string);
        })
    });
    group.finish();
}

criterion_group!(benches, bench_noop_sink, bench_influx_sink);
criterion_main!(benches);
