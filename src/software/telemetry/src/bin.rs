#[macro_use]
extern crate log;

use std::sync::mpsc::{Receiver, Sender, TryRecvError};

use telemetry::structures::*;
use telemetry::state::{TelemetryState};
use telemetry::*;

fn main() {
    env_logger::init();

    let mut telemetry_state = TelemetryState::new();

    if let Some(port_id) = std::env::args().nth(1) {
        if !port_id.is_empty() {
            let (tx, rx): (Sender<TelemetryMessage>, Receiver<TelemetryMessage>) =
                std::sync::mpsc::channel();
            std::thread::spawn(move || {
                gather_telemetry(&port_id, tx);
            });
            loop {
                match rx.try_recv() {
                    Ok(msg) => {
                        telemetry_state.push(msg);
                        telemetry_state.display();
                    }
                    Err(TryRecvError::Empty) => {
                        std::thread::sleep(std::time::Duration::from_millis(10));
                    }
                    Err(TryRecvError::Disconnected) => {
                        panic!("Channel to serial port thread was closed");
                    }
                }
            }
        } else {
            help();
        }
    } else {
        help();
    }
}

fn help() {
    error!("You need to specify a serial port address as first argument");
}
