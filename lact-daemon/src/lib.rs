#![warn(clippy::pedantic)]
#![allow(clippy::missing_panics_doc)]

mod config;
mod fork;
mod server;
mod socket;
mod suspend;

use anyhow::Context;
use config::Config;
use futures::future::select_all;
use server::{handle_stream, handler::Handler, Server};
use std::str::FromStr;
use std::{os::unix::net::UnixStream as StdUnixStream, time::Duration};
use tokio::{
    runtime,
    signal::unix::{signal, SignalKind},
    task::LocalSet,
};
use tracing::{debug, debug_span, info, warn, Instrument, Level};

pub use server::system::MODULE_CONF_PATH;

const MIN_SYSTEM_UPTIME_SECS: f32 = 10.0;
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

        ensure_sufficient_uptime().await;

        LocalSet::new()
            .run_until(async move {
                let server = Server::new(config).await?;
                let handler = server.handler.clone();

                tokio::task::spawn_local(listen_exit_signals(handler.clone()));
                tokio::task::spawn_local(suspend::listen_events(handler));
                server.run().await;
                Ok(())
            })
            .await
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
        LocalSet::new()
            .run_until(async move {
                let config = Config::default();
                let handler = Handler::new(config).await?;
                let stream = stream.try_into()?;

                handle_stream(stream, handler).await
            })
            .await
    })
}

async fn listen_exit_signals(handler: Handler) {
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

async fn ensure_sufficient_uptime() {
    match get_uptime() {
        Ok(current_uptime) => {
            debug!("current system uptime: {current_uptime:.1}s");

            let diff = MIN_SYSTEM_UPTIME_SECS - current_uptime;
            if diff > 0.0 {
                info!("service started too early, waiting {diff:.1} seconds");

                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                tokio::time::sleep(Duration::from_millis((diff * 1000.0) as u64)).await;
            }
        }
        Err(err) => {
            warn!("could not get system uptime: {err:#?}");
        }
    }
}

fn get_uptime() -> anyhow::Result<f32> {
    let raw_uptime = std::fs::read_to_string("/proc/uptime").context("Could not read uptime")?;
    raw_uptime
        .split_whitespace()
        .next()
        .context("Could not parse the uptime file")?
        .parse()
        .context("Invalid uptime value")
}
