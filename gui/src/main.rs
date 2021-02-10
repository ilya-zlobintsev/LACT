use std::thread;

use app::App;
use daemon::{daemon_connection::DaemonConnection, Daemon};
use gtk::*;

mod app;

fn main() {
    env_logger::init();
    if gtk::init().is_err() {
        panic!("Cannot initialize GTK");
    }

    let app = App::new(connect_daemon());

    app.run().unwrap();
}

fn connect_daemon() -> DaemonConnection {
    match DaemonConnection::new() {
        Ok(connection) => {
            println!("Connection to daemon established");
            connection
        }
        Err(e) => {
            println!("Error {:?} connecting to daemon", e);
            println!("Starting unprivileged daemon instance");

            thread::spawn(move || {
                let daemon = Daemon::new(true);
                daemon.listen();
            });

            let dialog = MessageDialog::new(
                None::<&gtk::Window>,
                DialogFlags::empty(),
                gtk::MessageType::Warning,
                gtk::ButtonsType::Ok,
                "Unable to connect to daemon. Running in unprivileged mode.",
            );

            dialog.run();
            dialog.close();

            DaemonConnection::new().unwrap()
        }
    }
}
