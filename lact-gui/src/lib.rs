mod app;

use anyhow::anyhow;
use app::App;
use lact_client::DaemonClient;
use std::os::unix::net::UnixStream;
use tracing::{error, info, metadata::LevelFilter};
use tracing_subscriber::EnvFilter;

pub fn run() -> anyhow::Result<()> {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();
    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    if let Err(err) = gtk::init() {
        return Err(anyhow!("Cannot initialize GTK: {err}"));
    }

    let connection = create_connection()?;
    let app = App::new(connection);

    app.run()
}

fn create_connection() -> anyhow::Result<DaemonClient> {
    match DaemonClient::connect() {
        Ok(connection) => Ok(connection),
        Err(err) => {
            info!("Could not connect to socket: {err}");
            info!("Using a local daemon");

            let (server_stream, client_stream) = UnixStream::pair()?;

            std::thread::spawn(move || {
                if let Err(err) = lact_daemon::run_embedded(server_stream) {
                    error!("Builtin daemon error: {err}");
                }
            });

            DaemonClient::from_stream(client_stream, true)
        }
    }
}
