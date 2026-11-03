//! Manages the tray icon process, which is `lact tray`.
//!
//! The tray icon runs in its own process, so that closing the GUI releases all of its memory and
//! nothing of the GUI keeps running in the background.
//!
//! The running tray is located through the D-Bus name it owns, which the message bus releases
//! automatically when the process exits, so no state has to be kept on disk.

use crate::APP_ID;
use anyhow::Context as _;
use gtk::{
    gio,
    glib::{self, variant::ToVariant as _},
};
use lact_schema::{TRAY_BUS_NAME, TRAY_SUBCOMMAND};
use nix::{
    sys::signal::{Signal, kill},
    unistd::Pid,
};
use std::{
    fs, io,
    os::unix::process::CommandExt as _,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};
use tracing::{debug, warn};

const AUTOSTART_FILE: &str = "io.github.ilya_zlobintsev.LACT.Tray.desktop";
/// Autostart entry, with the command line that launches the tray as a placeholder.
const AUTOSTART_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../res/io.github.ilya_zlobintsev.LACT.Tray.desktop.in"
));
const EXEC_PLACEHOLDER: &str = "@EXEC@";
const STOP_TIMEOUT: Duration = Duration::from_millis(1000);
const DBUS_TIMEOUT: i32 = 1000;

/// Starts the tray process if it is not running yet.
pub fn start() -> anyhow::Result<()> {
    if is_running() {
        debug!("the tray icon process is already running");
        return Ok(());
    }

    let binary =
        std::env::current_exe().context("Could not determine the path of the executable")?;
    debug!("starting the tray icon process: {}", binary.display());

    let mut command = Command::new(binary);
    command
        .arg(TRAY_SUBCOMMAND)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    // SAFETY: setsid only creates a new session for the child process, it does not access any
    // shared state. This detaches the tray so that it outlives the GUI.
    unsafe {
        command.pre_exec(|| {
            nix::unistd::setsid()
                .map(|_| ())
                .map_err(|err| io::Error::from_raw_os_error(err as i32))
        });
    }

    command
        .spawn()
        .context("Could not start the tray icon process")?;

    Ok(())
}

/// Stops the tray process, if it is running.
pub fn stop() {
    let Some(pid) = tray_pid() else {
        return;
    };

    debug!("stopping the tray icon process (pid {pid})");
    if let Err(err) = kill(Pid::from_raw(pid), Signal::SIGTERM) {
        warn!("could not stop the tray icon process: {err}");
        return;
    }

    let start = Instant::now();
    while is_running() && start.elapsed() < STOP_TIMEOUT {
        thread::sleep(Duration::from_millis(20));
    }

    // Reaps the tray if this GUI started it, so that it does not linger as a zombie. Fails with
    // `ECHILD` when the tray was started by someone else (e.g. the autostart entry), which is fine.
    let _ = nix::sys::wait::waitpid(
        Pid::from_raw(pid),
        Some(nix::sys::wait::WaitPidFlag::WNOHANG),
    );
}

/// Whether the tray process is currently running.
pub fn is_running() -> bool {
    tray_pid().is_some()
}

/// Resolves the pid of the tray process through the D-Bus name it owns.
fn tray_pid() -> Option<i32> {
    let connection = gio::bus_get_sync(gio::BusType::Session, gio::Cancellable::NONE).ok()?;

    let reply = connection
        .call_sync(
            Some("org.freedesktop.DBus"),
            "/org/freedesktop/DBus",
            "org.freedesktop.DBus",
            "GetConnectionUnixProcessID",
            Some(&(TRAY_BUS_NAME,).to_variant()),
            Some(glib::VariantTy::new("(u)").ok()?),
            gio::DBusCallFlags::NONE,
            DBUS_TIMEOUT,
            gio::Cancellable::NONE,
        )
        // Fails with `NameHasNoOwner` when the tray is not running.
        .ok()?;

    let (pid,) = reply.get::<(u32,)>()?;

    i32::try_from(pid).ok()
}

/// Creates or removes the autostart entry which launches the tray on login.
pub fn set_autostart(enabled: bool) -> anyhow::Result<()> {
    let path = autostart_path()?;

    if enabled {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)
                .with_context(|| format!("Could not create {}", dir.display()))?;
        }
        fs::write(&path, autostart_entry()?).with_context(|| {
            format!("Could not write the autostart entry at {}", path.display())
        })?;
        debug!("wrote the autostart entry at {}", path.display());
    } else if path.exists() {
        fs::remove_file(&path).with_context(|| format!("Could not remove {}", path.display()))?;
        debug!("removed the autostart entry at {}", path.display());
    }

    Ok(())
}

fn autostart_entry() -> anyhow::Result<String> {
    let binary =
        std::env::current_exe().context("Could not determine the path of the executable")?;
    // A desktop entry is text, so a path which cannot be represented in it is an error instead of
    // being silently mangled.
    let binary = binary
        .to_str()
        .context("The path of the executable is not valid UTF-8")?;

    let exec = if is_flatpak() {
        // Autostart entries are executed outside of the sandbox.
        let command = Path::new(binary)
            .file_name()
            .context("Could not determine the name of the executable")?
            .to_str()
            .context("The name of the executable is not valid UTF-8")?;
        format!("flatpak run --command={command} {APP_ID} {TRAY_SUBCOMMAND}")
    } else {
        format!("{binary} {TRAY_SUBCOMMAND}")
    };

    Ok(AUTOSTART_TEMPLATE.replace(EXEC_PLACEHOLDER, &exec))
}

fn is_flatpak() -> bool {
    Path::new("/.flatpak-info").exists()
}

fn autostart_path() -> anyhow::Result<PathBuf> {
    let config_dir = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .context("Could not determine the config directory")?;

    Ok(config_dir.join("autostart").join(AUTOSTART_FILE))
}
