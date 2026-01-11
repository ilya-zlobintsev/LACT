use crate::app::graphs_window::stat::StatType;
use serde::{Deserialize, Deserializer, Serialize};
use std::{collections::HashMap, env, fs, path::PathBuf};
use tracing::{debug, error};

pub const MIN_STATS_POLL_INTERVAL_MS: i64 = 250;
pub const MAX_STATS_POLL_INTERVAL_MS: i64 = 5000;

#[derive(Serialize, Deserialize)]
pub struct UiConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_gpu: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plots_time_period: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plots_per_row: Option<u64>,
    #[serde(
        default = "default_stats_poll_interval",
        deserialize_with = "deserialize_poll_interval"
    )]
    pub stats_poll_interval_ms: i64,
    #[serde(default)]
    pub gpus: HashMap<String, UiGpuConfig>,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            selected_gpu: None,
            plots_time_period: None,
            plots_per_row: None,
            stats_poll_interval_ms: default_stats_poll_interval(),
            gpus: HashMap::new(),
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct UiGpuConfig {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub plots: Vec<Vec<StatType>>,
}

impl UiConfig {
    pub fn edit(&mut self, f: impl FnOnce(&mut Self)) {
        f(self);
        self.save();
    }

    pub fn save(&self) {
        let path = config_path();
        debug!("saving config to {}", path.display());
        let config_dir = path.parent().unwrap();
        if !config_dir.exists() {
            if let Err(err) = fs::create_dir_all(config_dir) {
                error!("could not create config dir: {err}");
                return;
            }
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

fn deserialize_poll_interval<'de, D: Deserializer<'de>>(deserializer: D) -> Result<i64, D::Error> {
    let value = i64::deserialize(deserializer)?;
    Ok(value.clamp(MIN_STATS_POLL_INTERVAL_MS, MAX_STATS_POLL_INTERVAL_MS))
}
