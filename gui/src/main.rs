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

    let connection = connect_daemon();

    ask_for_online_update(&connection);

    let app = App::new(connection);

    app.run().unwrap();
}

fn ask_for_online_update(connection: &DaemonConnection) {
    let mut config = connection.get_config().unwrap();

    if let None = config.allow_online_update {
        log::trace!("Online access permission not configured! Showing the dialog");

        let diag = MessageDialog::new(
            None::<&Window>,
            DialogFlags::empty(),
            MessageType::Warning,
            ButtonsType::YesNo,
            "Do you wish to use the online database for GPU identification?",
        );
        match diag.run() {
            ResponseType::Yes => config.allow_online_update = Some(true),
            ResponseType::No => config.allow_online_update = Some(false),
            _ => unreachable!(),
        }
        diag.hide();

        connection.set_config(config).unwrap();
    }
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
