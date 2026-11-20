//! Tray icon of LACT, launched through the `lact tray` subcommand.
//!
//! The GUI exits when its window is closed, so that it does not keep any memory or run in the
//! background. This entry point takes over the tray icon instead: it does not initialize GTK and
//! reads the values shown in the menu and tooltip from the daemon.

mod config;
mod gui;
mod snapshot_ext;
mod tray;

use std::{io, sync::LazyLock, time::Duration};

use anyhow::Context as _;
use i18n_embed::fluent::{FluentLanguageLoader, fluent_language_loader};
use lact_client::DaemonClient;
use rust_embed::RustEmbed;
use tokio::{sync::mpsc, time::MissedTickBehavior};
use tracing::{debug, error, info};
use tracing_subscriber::EnvFilter;
use zbus::fdo::{RequestNameFlags, RequestNameReply};

use self::config::UiConfig;
use self::tray::{LactTray, TrayEvent};

pub use lact_schema::{APP_ID, TRAY_BUS_NAME};

/// How long to wait before trying to connect to the daemon again.
const CONNECT_RETRY_INTERVAL: Duration = Duration::from_secs(1);

pub static I18N: LazyLock<FluentLanguageLoader> =
    LazyLock::new(|| lact_schema::i18n::loader(fluent_language_loader!(), &Localizations, None));

#[derive(RustEmbed)]
#[folder = "i18n"]
struct Localizations;

/// Runs the tray icon until it is quit from its menu or the session bus goes away.
///
/// Uses a single-threaded runtime: the tray only answers StatusNotifierItem queries, there is no
/// background work to run in parallel.
#[tokio::main(flavor = "current_thread")]
pub async fn run() -> anyhow::Result<()> {
    init_logging();

    if !UiConfig::load().is_some_and(|config| config.tray_enabled()) {
        info!("the tray icon is disabled in the settings, exiting");
        return Ok(());
    }

    serve().await
}

async fn serve() -> anyhow::Result<()> {
    let bus = zbus::Connection::session()
        .await
        .context("Could not connect to the session bus")?;

    if !acquire_bus_name(&bus).await? {
        info!("the tray icon is already running, exiting");
        return Ok(());
    }

    // The tray only renders values and reports events: fetching and quitting are handled here.
    let (events, mut tray_events) = mpsc::unbounded_channel();
    let tray = LactTray::new(events);

    let handle = tray.spawn().await?;

    debug!("tray icon registered");

    // Created once: the client reconnects on its own after the daemon is restarted. The tray
    // starts with the session, which can be before the daemon has created its socket, so a failed
    // connect is retried instead of leaving the item permanently without data.
    let daemon = loop {
        match DaemonClient::connect().await {
            Ok(client) => break client,
            Err(err) => {
                debug!("could not connect to the daemon: {err:#}");
                tokio::time::sleep(CONNECT_RETRY_INTERVAL).await;
            }
        }
    };

    // The host gives no reliable signal for when the item is shown or hidden, so the values are
    // polled for the whole lifetime of the process, at the interval the main window uses.
    let mut refresh_interval =
        tokio::time::interval(UiConfig::load().unwrap_or_default().stats_poll_interval());
    refresh_interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

    // `interval` fires its first tick right away, which would repeat the read that was just done.
    refresh_interval.reset();

    // Serves until the session bus goes away (logout or bus restart) or the user quits from the
    // tray menu. Reads are awaited here instead of being spawned: `DaemonClient` is not `Send`.
    loop {
        tokio::select! {

            () = bus.closed() => break,
            // Matches `Some` only: the spawned item holds a sender of this channel, so it cannot
            // close while the loop runs - `recv()` never yields `None` here.
            Some(event) = tray_events.recv() => match event {
                // Publishing the values is also what re-emits the properties. Reading them right
                // away makes a freshly opened item show current values instead of the last tick.
                TrayEvent::Show => handle.refresh(&daemon).await,
                TrayEvent::SetProfile { name, auto_switch } => {
                    debug!("switching to the {name:?} profile");
                    if let Err(err) = daemon.set_profile(name, auto_switch).await {
                        error!("could not switch the profile: {err:#}");
                    }

                    // Reads back the profile which actually became active.
                    handle.refresh(&daemon).await;
                }
                TrayEvent::SetAutoSwitch(enabled) => {
                    debug!("setting profile auto switching to {enabled}");
                    // The active profile is read again instead of taken from the last published
                    // snapshot: `set_profile` always carries both values, so a stale name would
                    // silently switch the profile while only the setting was meant to change.
                    match daemon.list_profiles(false).await {
                        Ok(info) => {
                            if let Err(err) =
                                daemon.set_profile(info.current_profile, enabled).await
                            {
                                error!("could not change profile auto switching: {err:#}");
                            }

                            handle.refresh(&daemon).await;
                        }
                        Err(err) => error!("could not read the profiles: {err:#}"),
                    }
                }
                TrayEvent::ShowWindow => {
                    if let Err(err) = gui::spawn() {
                        error!("could not open the GUI: {err:#}");
                    }
                }
                TrayEvent::Quit => {
                    gui::quit().await;
                    break;
                }
            },
            _ = refresh_interval.tick() => handle.refresh(&daemon).await,
        }
    }

    Ok(())
}

/// Claims the tray's D-Bus name, returning `false` if another instance already owns it.
async fn acquire_bus_name(connection: &zbus::Connection) -> anyhow::Result<bool> {
    // Calls `RequestName` directly instead of `Connection::request_name_with_flags`, which tracks
    // name ownership and warns about the missing object server - neither of which this process
    // needs, the bus releases the name on its own once the connection is gone.
    let flags = RequestNameFlags::DO_NOT_QUEUE.bits();

    let reply = connection
        .call_method(
            Some("org.freedesktop.DBus"),
            "/org/freedesktop/DBus",
            Some("org.freedesktop.DBus"),
            "RequestName",
            &(TRAY_BUS_NAME, flags),
        )
        .await
        .context("Could not acquire the tray D-Bus name")?
        .body()
        .deserialize::<RequestNameReply>()
        .context("Could not parse the RequestName reply")?;

    Ok(matches!(reply, RequestNameReply::PrimaryOwner))
}

fn init_logging() {
    let filter = EnvFilter::builder()
        .with_default_directive(tracing::Level::INFO.into())
        .from_env_lossy();

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(io::stderr)
        .init();
}
