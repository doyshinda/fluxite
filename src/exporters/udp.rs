use log::{error, trace};
use metrics_core::{Builder, Drain, Observe, Observer};
use std::{net::UdpSocket, thread, time::Duration};

/// Exports metrics over UDP.
pub struct UdpExporter<C, B>
where
    B: Builder,
{
    controller: C,
    observer: B::Output,
    sock: UdpSocket,
    interval: Duration,
    endpoint: String,
}

impl<C, B> UdpExporter<C, B>
where
    B: Builder,
    B::Output: Drain<String> + Observer,
    C: Observe,
{
    pub fn new(controller: C, builder: B, interval: Duration, endpoint: &str) -> Self {
        UdpExporter {
            controller,
            observer: builder.build(),
            sock: UdpSocket::bind("0.0.0.0:0").expect("failed to bind host socket"),
            interval,
            endpoint: endpoint.to_string(),
        }
    }

    pub fn run(&mut self) {
        loop {
            thread::sleep(self.interval);

            self.turn();
        }
    }

    /// Run this exporter, logging output only once.
    fn turn(&mut self) {
        self.controller.observe(&mut self.observer);
        let output = self.observer.drain();
        let size = output.len();
        if let Err(e) = self.sock.send_to(&output.into_bytes(), &self.endpoint) {
            error!("Error sending on socket: {:?}", e);
        } else {
            trace!("Sent {} bytes", size);
        }
    }
}
