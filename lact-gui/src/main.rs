mod app;

use anyhow::{anyhow, Context};
use app::App;
use lact_client::DaemonClient;
use tracing::metadata::LevelFilter;
use tracing_subscriber::EnvFilter;

fn main() -> anyhow::Result<()> {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();
    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    if let Err(err) = gtk::init() {
        return Err(anyhow!("Cannot initialize GTK: {err}"));
    }

    let connection = DaemonClient::connect().context("Could not connect to daemon")?;

    let app = App::new(connection);

    app.run()?;

    Ok(())
}
