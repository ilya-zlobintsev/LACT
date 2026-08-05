use crate::app::{
    components::gpu_stats_section::{GpuStat, GpuStatDisplay},
    graphs_window::stat::StatType,
    utils::{color_scheme::AppColorScheme, styles::AppTheme},
};
use serde::{Deserialize, Deserializer, Serialize};
use serde_norway::Value;
use serde_with::skip_serializing_none;
use std::{
    collections::{HashMap, HashSet},
    env, fs,
    path::PathBuf,
};
use tracing::{debug, error};

pub const MIN_STATS_POLL_INTERVAL_MS: i64 = 250;
pub const MAX_STATS_POLL_INTERVAL_MS: i64 = 5000;

#[skip_serializing_none]
#[derive(Serialize, Deserialize)]
pub struct UiConfig {
    #[serde(default = "default_tab")]
    pub selected_tab: String,
    pub selected_gpu: Option<String>,
    pub plots_time_period: Option<u64>,
    pub plots_per_row: Option<u64>,
    #[serde(
        default = "default_stats_poll_interval",
        deserialize_with = "deserialize_poll_interval"
    )]
    pub stats_poll_interval_ms: i64,
    #[serde(default)]
    pub gpus: HashMap<String, UiGpuConfig>,
    #[serde(default)]
    pub theme: AppTheme,
    #[serde(default)]
    pub color_scheme: AppColorScheme,
    pub window_size: Option<WindowSize>,
    #[serde(default, deserialize_with = "deserialize_stats_layout")]
    pub stats_layout: HashMap<StatsPage, StatsLayout>,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            selected_tab: default_tab(),
            selected_gpu: None,
            plots_time_period: None,
            plots_per_row: None,
            stats_poll_interval_ms: default_stats_poll_interval(),
            gpus: HashMap::new(),
            theme: AppTheme::Automatic,
            color_scheme: AppColorScheme::default(),
            window_size: None,
            stats_layout: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StatsPage {
    OcPage,
    ThermalsPage,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub struct StatEntry {
    pub stat: GpuStat,
    pub display: GpuStatDisplay,
    pub enabled: bool,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize)]
pub struct StatsLayout(pub Vec<StatEntry>);

impl StatsPage {
    pub fn default_layout(self) -> StatsLayout {
        let entries = GpuStat::ALL
            .iter()
            .copied()
            .map(|stat| StatEntry {
                stat,
                display: stat.default_display(),
                enabled: match self {
                    Self::OcPage => true,
                    Self::ThermalsPage => matches!(
                        stat,
                        GpuStat::Throttling
                            | GpuStat::Temperature
                            | GpuStat::PowerUsage
                            | GpuStat::FanSpeed
                    ),
                },
            })
            .collect();
        StatsLayout(entries)
    }

    fn merge_layout(self, stored: Option<StatsLayout>) -> StatsLayout {
        let defaults = self.default_layout();
        let Some(stored) = stored else {
            return defaults;
        };

        let mut seen = HashSet::new();
        let mut entries = stored
            .0
            .into_iter()
            .filter_map(|mut entry| {
                if entry.stat == GpuStat::Unknown || !seen.insert(entry.stat) {
                    return None;
                }
                if !entry.stat.supported_displays().contains(&entry.display) {
                    entry.display = entry.stat.default_display();
                }
                Some(entry)
            })
            .collect::<Vec<_>>();

        for (default_index, default_entry) in defaults.0.into_iter().enumerate() {
            if seen.insert(default_entry.stat) {
                let insert_at = entries
                    .iter()
                    .position(|entry| {
                        GpuStat::ALL
                            .iter()
                            .position(|stat| *stat == entry.stat)
                            .is_some_and(|index| index > default_index)
                    })
                    .unwrap_or(entries.len());
                entries.insert(insert_at, default_entry);
            }
        }

        StatsLayout(entries)
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct WindowSize {
    pub width: i32,
    pub height: i32,
}

#[derive(Default, Serialize, Deserialize)]
pub struct UiGpuConfig {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub plots: Vec<Vec<StatType>>,
}

impl UiConfig {
    pub fn stats_layout_for(&self, page: StatsPage) -> StatsLayout {
        page.merge_layout(self.stats_layout.get(&page).cloned())
    }

    pub fn edit(&mut self, f: impl FnOnce(&mut Self)) {
        f(self);
        self.save();
    }

    pub fn save(&self) {
        let path = config_path();
        debug!("saving config to {}", path.display());
        let config_dir = path.parent().unwrap();
        if !config_dir.exists()
            && let Err(err) = fs::create_dir_all(config_dir)
        {
            error!("could not create config dir: {err}");
            return;
        }

        let raw_config = serde_norway::to_string(self).unwrap();
        if let Err(err) = fs::write(path, raw_config) {
            error!("could not write config: {err}");
        }
    }

    pub fn load() -> Option<Self> {
        let path = config_path();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(raw_config) => match serde_norway::from_str::<Self>(&raw_config) {
                    Ok(config) => Some(config),
                    Err(err) => {
                        error!("could not parse config: {err}");
                        None
                    }
                },
                Err(err) => {
                    error!("could not read config: {err}");
                    None
                }
            }
        } else {
            None
        }
    }
}

fn config_path() -> PathBuf {
    let config_dir = PathBuf::from(env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
        let home = env::var("HOME").expect("$HOME variable is not set");
        format!("{home}/.config")
    }));
    config_dir.join("lact").join("ui.yaml")
}

fn default_stats_poll_interval() -> i64 {
    500
}

fn deserialize_stats_layout<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<HashMap<StatsPage, StatsLayout>, D::Error> {
    let stored = HashMap::<Value, Vec<Value>>::deserialize(deserializer)?;
    Ok(stored
        .into_iter()
        .filter_map(|(page, entries)| {
            let page = StatsPage::deserialize(page)
                .inspect_err(|err| debug!("ignoring stats layout: {err}"))
                .ok()?;
            let entries = entries
                .into_iter()
                .filter_map(|entry| {
                    StatEntry::deserialize(entry)
                        .inspect_err(|err| debug!("ignoring stat entry: {err}"))
                        .ok()
                })
                .collect();
            Some((page, StatsLayout(entries)))
        })
        .collect())
}

fn deserialize_poll_interval<'de, D: Deserializer<'de>>(deserializer: D) -> Result<i64, D::Error> {
    let value = i64::deserialize(deserializer)?;
    Ok(value.clamp(MIN_STATS_POLL_INTERVAL_MS, MAX_STATS_POLL_INTERVAL_MS))
}

fn default_tab() -> String {
    "info_page".to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_layouts_match_existing_pages() {
        let oc = StatsPage::OcPage.default_layout();
        assert_eq!(oc.0.len(), GpuStat::ALL.len());
        assert!(oc.0.iter().all(|entry| entry.enabled));
        assert_eq!(
            oc.0.iter().map(|entry| entry.stat).collect::<Vec<_>>(),
            GpuStat::ALL
        );

        let thermals = StatsPage::ThermalsPage.default_layout();
        let enabled = thermals
            .0
            .iter()
            .filter(|entry| entry.enabled)
            .map(|entry| entry.stat)
            .collect::<Vec<_>>();
        assert_eq!(
            enabled,
            [
                GpuStat::Throttling,
                GpuStat::Temperature,
                GpuStat::PowerUsage,
                GpuStat::FanSpeed,
            ]
        );
    }

    #[test]
    fn stored_layout_is_normalized_and_completed() {
        let stored = StatsLayout(vec![
            StatEntry {
                stat: GpuStat::DeviceName,
                display: GpuStatDisplay::LevelBar,
                enabled: false,
            },
            StatEntry {
                stat: GpuStat::Temperature,
                display: GpuStatDisplay::Text,
                enabled: true,
            },
            StatEntry {
                stat: GpuStat::DeviceName,
                display: GpuStatDisplay::Text,
                enabled: true,
            },
        ]);

        let merged = StatsPage::OcPage.merge_layout(Some(stored));
        assert_eq!(merged.0.len(), GpuStat::ALL.len());
        assert_eq!(merged.0[0].stat, GpuStat::DeviceName);
        assert_eq!(merged.0[0].display, GpuStatDisplay::Text);
        assert!(!merged.0[0].enabled);
        assert_eq!(merged.0[4].stat, GpuStat::Temperature);
    }

    #[test]
    fn unknown_stats_are_dropped_from_the_layout() {
        let config: UiConfig = serde_norway::from_str(
            "stats_layout:\n  oc-page:\n  - stat: future-stat\n    display: text\n    enabled: true\n  - stat: gpu-usage\n    display: level-bar\n    enabled: true\n",
        )
        .unwrap();

        let layout = &config.stats_layout[&StatsPage::OcPage];
        assert_eq!(layout.0.len(), 2);
        assert_eq!(
            StatsPage::OcPage.merge_layout(Some(layout.clone())).0[0].stat,
            GpuStat::DeviceName
        );
    }

    #[test]
    fn broken_stats_layout_does_not_discard_the_config() {
        let config: UiConfig = serde_norway::from_str(
            "selected_tab: oc_page\nstats_layout:\n  future-page:\n  - stat: gpu-usage\n    display: text\n    enabled: true\n  oc-page:\n  - stat: gpu-usage\n  - stat: fan-speed\n    display: text\n    enabled: false\n",
        )
        .unwrap();

        assert_eq!(config.selected_tab, "oc_page");
        assert_eq!(config.stats_layout.len(), 1);
        let layout = &config.stats_layout[&StatsPage::OcPage];
        assert_eq!(layout.0.len(), 1);
        assert_eq!(layout.0[0].stat, GpuStat::FanSpeed);
    }

    #[test]
    fn old_config_defaults_to_no_stored_layouts() {
        let config: UiConfig = serde_norway::from_str("{}\n").unwrap();
        assert!(config.stats_layout.is_empty());
    }
}
