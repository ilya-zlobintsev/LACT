use std::thread;

use daemon::{daemon_connection::DaemonConnection, Daemon};
use signal_hook::consts::{SIGINT, SIGTERM};
use signal_hook::iterator::Signals;

fn main() {
    env_logger::init();
    let d = Daemon::new(false);
    let mut signals = Signals::new(&[SIGTERM, SIGINT]).unwrap();

    thread::spawn(move || {
        for _ in signals.forever() {
            log::info!("Shutting down");
            let d = DaemonConnection::new().unwrap();
            d.shutdown();
        }
    });

    d.listen();
}
