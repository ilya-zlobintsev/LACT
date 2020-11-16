use std::thread;

use daemon::{Daemon, daemon_connection::DaemonConnection};
use signal_hook::{SIGINT, SIGTERM, iterator::Signals};

fn main() {
    env_logger::init();
    let d = Daemon::new(false);
    let signals = Signals::new(&[SIGTERM, SIGINT]).unwrap();
    
    thread::spawn(move || {
        for _ in signals.forever() {
            log::info!("Shutting down");
            let d = DaemonConnection::new().unwrap();
            d.shutdown();
        }
    });
    
    d.listen();
}
