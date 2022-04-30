use fluxite::{init_exporter, MetricsConfig, SinkType};
use fluxite_macro::count;
use std::process::Command;
use std::{thread, time::Duration};

fn hostname() -> String {
    let out = Command::new("hostname")
        .output()
        .expect("Unable to retrieve hostname");
    let hostname = std::str::from_utf8(&out.stdout).unwrap();
    hostname.replace('\n', "")
}

fn main() {
    let config = MetricsConfig {
        endpoint: "127.0.0.1:2115".to_string(),
        sink_type: SinkType::Influx,
        prefix: None,
        interval: None,
    };
    init_exporter(&config).unwrap();
    let my_string = hostname();

    for x in 0..5 {
        count!("foo", x, "hostname" => my_string);
        thread::sleep(Duration::from_secs(5));
        count!("foo", 1,);
    }
}
