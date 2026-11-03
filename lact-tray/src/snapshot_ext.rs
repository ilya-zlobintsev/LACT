//! Extension trait adding the tray's [`Snapshot`] read to [`lact_client::DaemonClient`].
//!
//! The daemon client is kept open between reads: the tray only asks for values when it is shown,
//! so opening a new connection for every read would spend most of its life in the handshake.

use std::time::Duration;

use anyhow::Context as _;
use lact_client::DaemonClient;
use lact_schema::DeviceStats;

use super::config::UiConfig;

/// Upper bound for a single read, so that a hung daemon cannot keep the tray waiting for it.
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(2);

/// Basic GPU information displayed in the tray tooltip and menu.
#[derive(Clone, Debug, Default)]
pub struct Snapshot {
    pub gpu_name: Option<String>,
    pub temperature: Option<f32>,
    pub usage_percent: Option<u8>,
    pub power_watts: Option<f64>,
}

impl Snapshot {
    fn new(gpu_name: Option<String>, stats: &DeviceStats) -> Self {
        Self {
            gpu_name,
            temperature: max_primary_temperature(stats),
            usage_percent: stats.busy_percent,
            power_watts: stats.power.current.or(stats.power.average),
        }
    }
}

/// Returns the highest reported primary temperature.
///
/// Several sensors can be marked as primary (e.g. edge and junction on AMD), the highest one is
/// the most interesting value to show in the tray.
fn max_primary_temperature(stats: &DeviceStats) -> Option<f32> {
    let mut primary = None;
    let mut any = None;

    for entry in stats.temps.values() {
        if let Some(current) = entry.value.current {
            any = Some(any.map_or(current, |value: f32| value.max(current)));

            if entry.primary {
                primary = Some(primary.map_or(current, |value: f32| value.max(current)));
            }
        }
    }

    primary.or(any)
}

/// Reads the values displayed by the tray.
pub trait SnapshotExt {
    /// Reads the values of the GPU selected in the configuration.
    async fn snapshot(&self) -> anyhow::Result<Snapshot>;
}

impl SnapshotExt for DaemonClient {
    async fn snapshot(&self) -> anyhow::Result<Snapshot> {
        match tokio::time::timeout(REQUEST_TIMEOUT, read(self)).await {
            Ok(result) => result,
            Err(_) => anyhow::bail!("Reading the GPU values timed out after {REQUEST_TIMEOUT:?}"),
        }
    }
}

async fn read(client: &DaemonClient) -> anyhow::Result<Snapshot> {
    let devices = client
        .list_devices()
        .await
        .context("Could not list the devices")?;

    // The configuration is read on every refresh, so that switching the GPU in the GUI is picked
    // up by the next read instead of requiring a tray restart.
    let config = UiConfig::load();
    let device = config
        .as_ref()
        .and_then(UiConfig::selected_gpu)
        .and_then(|id| devices.iter().find(|device| device.id == id))
        .or_else(|| devices.first())
        .context("The daemon reported no devices")?;

    let stats = client
        .get_device_stats(&device.id)
        .await
        .with_context(|| format!("Could not read the stats of '{}'", device.id))?;

    Ok(Snapshot::new(Some(device.to_string()), &stats))
}

#[cfg(test)]
mod tests {
    use super::*;
    use amdgpu_sysfs::hw_mon::Temperature;
    use indexmap::IndexMap;
    use lact_schema::TemperatureEntry;

    fn temperature_entry(current: f32, primary: bool) -> TemperatureEntry {
        TemperatureEntry {
            value: Temperature {
                crit: None,
                crit_hyst: None,
                current: Some(current),
            },
            primary,
            display_only: false,
        }
    }

    #[test]
    fn uses_highest_primary_temperature() {
        let mut stats = DeviceStats::default();
        stats.temps = IndexMap::from([
            ("edge".to_owned(), temperature_entry(55.0, true)),
            ("junction".to_owned(), temperature_entry(67.4, true)),
            ("mem".to_owned(), temperature_entry(80.0, false)),
        ]);

        let snapshot = Snapshot::new(None, &stats);
        assert_eq!(snapshot.temperature, Some(67.4));
    }

    #[test]
    fn falls_back_to_non_primary_temperatures() {
        let mut stats = DeviceStats::default();
        stats.temps = IndexMap::from([("gpu".to_owned(), temperature_entry(51.0, false))]);

        assert_eq!(Snapshot::new(None, &stats).temperature, Some(51.0));
    }

    #[test]
    fn uses_average_power_as_fallback() {
        let mut stats = DeviceStats::default();
        stats.power.average = Some(80.0);

        assert_eq!(Snapshot::new(None, &stats).power_watts, Some(80.0));
    }
}
