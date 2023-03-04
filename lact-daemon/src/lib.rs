#![warn(clippy::pedantic)]

mod config;
mod fork;
mod server;
mod socket;

use anyhow::Context;
use config::Config;
use futures::future::select_all;
use server::{handle_stream, handler::Handler, Server};
use std::os::unix::net::UnixStream as StdUnixStream;
use std::str::FromStr;
use tokio::{
    runtime,
    signal::unix::{signal, SignalKind},
};
use tracing::{debug_span, info, Instrument, Level};

pub use server::system::MODULE_CONF_PATH;

const SHUTDOWN_SIGNALS: [SignalKind; 4] = [
    SignalKind::terminate(),
    SignalKind::interrupt(),
    SignalKind::quit(),
    SignalKind::hangup(),
];

/// Run the daemon, binding to the default socket.
///
/// # Errors
/// Returns an error when the daemon cannot initialize.
pub fn run() -> anyhow::Result<()> {
    let rt = runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Could not initialize tokio runtime");
    rt.block_on(async {
        let config = Config::load_or_create()?;

        let max_level = Level::from_str(&config.daemon.log_level).context("Invalid log level")?;
        tracing_subscriber::fmt().with_max_level(max_level).init();

        let server = Server::new(config).await?;
        let handler = server.handler.clone();

        tokio::spawn(listen_shutdown(handler));
        server.run().await;
        Ok(())
    })
}

/// Run the daemon with a given `UnixStream`.
/// This will NOT bind to a socket by itself, and the daemon will only be accessible via the given stream.
///
/// # Errors
/// Returns an error when the daemon cannot initialize.
pub fn run_embedded(stream: StdUnixStream) -> anyhow::Result<()> {
    let rt = runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Could not initialize tokio runtime");
    rt.block_on(async {
        let config = Config::default();
        let handler = Handler::new(config).await?;
        let stream = stream.try_into()?;

        handle_stream(stream, handler).await
    })
}

async fn listen_shutdown(handler: Handler) {
    let mut signals = SHUTDOWN_SIGNALS
        .map(|signal_kind| signal(signal_kind).expect("Could not listen to shutdown signal"));
    let signal_futures = signals.iter_mut().map(|signal| Box::pin(signal.recv()));
    select_all(signal_futures).await;

    info!("cleaning up and shutting down...");
    async {
        handler.cleanup().await;
        socket::cleanup();
    }
    .instrument(debug_span!("shutdown_cleanup"))
    .await;
    std::process::exit(0);
}
