mod config;
mod fork;
mod server;
mod socket;

use anyhow::Context;
use config::Config;
use server::Server;
use std::str::FromStr;
use tokio::signal::ctrl_c;
use tracing::{warn, Level};

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let config = Config::load_or_create()?;

    let max_level = Level::from_str(&config.daemon.log_level).context("Invalid log level")?;
    tracing_subscriber::fmt().with_max_level(max_level).init();

    let server = Server::new(config).await?;
    let handler = server.handler.clone();

    tokio::spawn(async move {
        ctrl_c().await.expect("Could not listen to shutdown signal");

        for controller in handler.gpu_controllers.values() {
            if let Err(err) = controller.stop_fan_control().await {
                warn!("Could not stop fan control on shutdown: {err}");
            }
        }

        socket::cleanup();
        std::process::exit(0);
    });

    Ok(server.run().await)
}
