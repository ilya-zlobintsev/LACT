//! Starting the GUI, which is the `gui` subcommand of this same binary.

use std::{io, process::Stdio};

use anyhow::Context as _;
use lact_schema::{APP_ID, GUI_SUBCOMMAND};
use nix::{
    sys::signal::{Signal, kill},
    unistd::Pid,
};
use tokio::process::Command;
use tracing::{debug, warn};

/// Starts the GUI in its own session, so that it outlives the tray.
pub fn spawn() -> anyhow::Result<()> {
    let binary =
        std::env::current_exe().context("Could not determine the path of the executable")?;

    let mut command = Command::new(binary);
    command
        .arg(GUI_SUBCOMMAND)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    // SAFETY: setsid only creates a new session for the child process, it does not access any
    // shared state.
    unsafe {
        command.pre_exec(|| {
            nix::unistd::setsid()
                .map(|_| ())
                .map_err(|err| io::Error::from_raw_os_error(err as i32))
        });
    }

    let mut child = command.spawn().context("Could not spawn the GUI process")?;

    // The GUI outlives the click which opened it, so the wait has to happen in the background.
    // Awaiting the child (instead of dropping it) is what lets the runtime reap it once it exits,
    // otherwise every window opened from the menu would leave a zombie behind.
    tokio::spawn(async move {
        child.wait().await.ok();
    });

    Ok(())
}

/// Asks a running GUI to quit, if there is one.
pub async fn quit() {
    let Some(pid) = pid().await else {
        debug!("the GUI is not running");
        return;
    };

    debug!("asking the GUI to quit (pid {pid})");
    if let Err(err) = kill(Pid::from_raw(pid), Signal::SIGTERM) {
        warn!("could not signal the GUI process: {err}");
    }
}

/// Resolves the pid of the running GUI through the bus name it owns.
async fn pid() -> Option<i32> {
    let connection = zbus::Connection::session().await.ok()?;

    // Looked up directly instead of through `zbus::fdo::DBusProxy`, which needs the `proxy`
    // feature for this single call.
    let pid = connection
        .call_method(
            Some("org.freedesktop.DBus"),
            "/org/freedesktop/DBus",
            Some("org.freedesktop.DBus"),
            "GetConnectionUnixProcessID",
            &(APP_ID,),
        )
        .await
        .ok()?
        .body()
        .deserialize::<u32>()
        .ok()?;

    i32::try_from(pid).ok()
}
