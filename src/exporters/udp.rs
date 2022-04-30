use crossbeam_channel::Receiver;
use log::{error, trace};
use std::{
    net::UdpSocket,
    thread,
    time::{Duration, Instant},
};

/// Exports metrics over UDP.
pub struct UdpExporter {
    sock: UdpSocket,
    interval: Duration,
    endpoint: String,
    rx: Receiver<String>,
}

impl UdpExporter {
    pub fn new(interval: Duration, endpoint: String, rx: Receiver<String>) -> Self {
        UdpExporter {
            sock: UdpSocket::bind("0.0.0.0:0").expect("failed to bind host socket"),
            interval,
            endpoint,
            rx,
        }
    }

    pub fn run(&mut self) {
        let mut metrics: Vec<String> = Vec::new();
        loop {
            let start = Instant::now();
            metrics.clear();
            while let Some(message) = self.rx.try_iter().next() {
                metrics.push(message);
            }

            if !metrics.is_empty() {
                self.turn(metrics.join("\n"));
            }

            let elapsed = Instant::now() - start;
            if elapsed < self.interval {
                thread::sleep(self.interval - elapsed);
            }
        }
    }

    /// Run this exporter, logging output only once.
    fn turn(&mut self, output: String) {
        let size = output.len();
        if let Err(e) = self.sock.send_to(&output.into_bytes(), &self.endpoint) {
            error!("Error sending on socket: {:?}", e);
            return;
        }
        trace!("Sent {} bytes", size);
    }
}
