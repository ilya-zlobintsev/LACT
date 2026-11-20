//! Access to the parts of the GUI configuration that the tray icon needs.
//!
//! Only the relevant fields are deserialized, so that the tray does not depend on the GUI.

use lact_schema::{
    DEFAULT_STATS_POLL_INTERVAL_MS, MAX_STATS_POLL_INTERVAL_MS, MIN_STATS_POLL_INTERVAL_MS,
};
use serde::Deserialize;
use std::{env, fs, path::PathBuf, time::Duration};

/// The parts of the GUI configuration that the tray icon needs.
///
/// Read from disk on every use, so that changing a setting in the GUI is picked up without
/// restarting the tray.
#[derive(Debug, Default, Deserialize)]
pub struct UiConfig {
    selected_gpu: Option<String>,
    #[serde(default)]
    tray_icon: bool,
    stats_poll_interval_ms: Option<i64>,
}

impl UiConfig {
    /// Reads the current GUI configuration, if it has been written yet.
    pub fn load() -> Option<Self> {
        let path = config_path()?;
        let data = fs::read_to_string(path).ok()?;
        serde_norway::from_str(&data).ok()
    }

    /// Whether the tray icon is enabled in the GUI settings.
    pub fn tray_enabled(&self) -> bool {
        self.tray_icon
    }

    /// The GPU that is selected in the GUI, if any.
    pub fn selected_gpu(&self) -> Option<&str> {
        self.selected_gpu.as_deref()
    }

    /// How often the main window polls the stats.
    ///
    /// Clamped to the bounds the GUI uses, so that a hand-edited file cannot make the tray hammer
    /// the daemon.
    pub fn stats_poll_interval(&self) -> Duration {
        let interval = self
            .stats_poll_interval_ms
            .unwrap_or(DEFAULT_STATS_POLL_INTERVAL_MS)
            .clamp(MIN_STATS_POLL_INTERVAL_MS, MAX_STATS_POLL_INTERVAL_MS);

        Duration::from_millis(interval as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(yaml: &str) -> UiConfig {
        serde_norway::from_str(yaml).expect("test YAML is valid")
    }

    #[test]
    fn tray_is_disabled_by_default() {
        assert!(!parse("selected_gpu: ~").tray_enabled());
        assert!(parse("tray_icon: true").tray_enabled());
    }

    #[test]
    fn falls_back_to_the_default_poll_interval() {
        assert_eq!(
            parse("tray_icon: true").stats_poll_interval(),
            Duration::from_millis(DEFAULT_STATS_POLL_INTERVAL_MS as u64)
        );
    }

    #[test]
    fn clamps_the_poll_interval() {
        assert_eq!(
            parse("stats_poll_interval_ms: 10").stats_poll_interval(),
            Duration::from_millis(MIN_STATS_POLL_INTERVAL_MS as u64)
        );
        assert_eq!(
            parse("stats_poll_interval_ms: 99999").stats_poll_interval(),
            Duration::from_millis(MAX_STATS_POLL_INTERVAL_MS as u64)
        );
    }
}

fn config_path() -> Option<PathBuf> {
    let base = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;

    Some(base.join("lact").join("ui.yaml"))
}
