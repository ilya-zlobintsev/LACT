mod app;

use anyhow::{anyhow, Context};
use app::App;
use lact_client::{schema::args::GuiArgs, DaemonClient};
use std::os::unix::net::UnixStream;
use tracing::{error, info, metadata::LevelFilter};
use tracing_subscriber::EnvFilter;

const GUI_VERSION: &str = env!("CARGO_PKG_VERSION");
const APP_ID: &str = "io.github.lact-linux";

pub fn run(args: GuiArgs) -> anyhow::Result<()> {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .parse(args.log_level.unwrap_or_default())
        .context("Invalid log level")?;
    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    #[cfg(feature = "libadwaita")]
    if let Err(err) = adw::init() {
        return Err(anyhow!("Cannot initialize Libadwaita: {err}"));
    }

    #[cfg(not(feature = "libadwaita"))]
    if let Err(err) = gtk::init() {
        return Err(anyhow!("Cannot initialize GTK: {err}"));
    }

    let (connection, connection_err) = create_connection()?;
    let app = App::new(connection);

    app.run(connection_err)
}

fn create_connection() -> anyhow::Result<(DaemonClient, Option<anyhow::Error>)> {
    match DaemonClient::connect() {
        Ok(connection) => Ok((connection, None)),
        Err(err) => {
            info!("could not connect to socket: {err:#}");
            info!("using a local daemon");

            let (server_stream, client_stream) = UnixStream::pair()?;

            std::thread::spawn(move || {
                if let Err(err) = lact_daemon::run_embedded(server_stream) {
                    error!("Builtin daemon error: {err}");
                }
            });

            let client = DaemonClient::from_stream(client_stream, true)?;
            Ok((client, Some(err)))
        }
    }
}
