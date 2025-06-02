use crate::app::graphs_window::stat::StatType;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env, fs, path::PathBuf};
use tracing::{debug, error};

#[derive(Default, Serialize, Deserialize)]
pub struct UiConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_gpu: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plots_time_period: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plots_per_row: Option<u64>,
    #[serde(default)]
    pub gpus: HashMap<String, UiGpuConfig>,
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

        let raw_config = serde_yml::to_string(self).unwrap();
        if let Err(err) = fs::write(path, raw_config) {
            error!("could not write config: {err}");
        }
    }

    pub fn load() -> Option<Self> {
        let path = config_path();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(raw_config) => match serde_yml::from_str::<Self>(&raw_config) {
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
