use crate::server::gpu_controller::fan_control::FanCurve;
use anyhow::Context;
use nix::unistd::getuid;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env, fs, path::PathBuf};
use tracing::debug;

const FILE_NAME: &str = "config.yaml";

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Config {
    pub daemon: DaemonConfig,
    pub gpus: HashMap<String, GpuConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DaemonConfig {
    pub log_level: String,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            log_level: "info".to_owned(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct GpuConfig {
    pub fan_control_enabled: bool,
    pub fan_control_settings: Option<FanControlSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FanControlSettings {
    pub temperature_key: String,
    pub interval_ms: u64,
    pub curve: FanCurve,
}

impl Config {
    pub fn load() -> anyhow::Result<Option<Self>> {
        let path = get_path();
        if path.exists() {
            let raw_config = fs::read_to_string(path).context("Could not open config file")?;
            let config =
                serde_yaml::from_str(&raw_config).context("Could not deserialize config")?;
            Ok(Some(config))
        } else {
            let parent = path.parent().unwrap();
            fs::create_dir_all(parent)?;
            Ok(None)
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = get_path();
        debug!("Saving config to {path:?}");
        let raw_config = serde_yaml::to_string(self)?;
        fs::write(path, raw_config).context("Could not write config")
    }

    pub fn load_or_create() -> anyhow::Result<Self> {
        match Config::load()? {
            Some(config) => Ok(config),
            None => {
                let config = Config::default();
                config.save()?;
                Ok(config)
            }
        }
    }
}

fn get_path() -> PathBuf {
    let uid = getuid();
    if uid.is_root() {
        PathBuf::from("/etc/lact").join(FILE_NAME)
    } else {
        let config_dir = PathBuf::from(env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
            let home = env::var("HOME").expect("$HOME variable is not set");
            format!("{home}/.config")
        }));
        config_dir.join("lact").join(FILE_NAME)
    }
}

#[cfg(test)]
mod tests {
    use super::{Config, DaemonConfig, FanControlSettings, GpuConfig};
    use crate::server::gpu_controller::fan_control::FanCurve;

    #[test]
    fn serde_de_full() {
        let config = Config {
            daemon: DaemonConfig::default(),
            gpus: [(
                "my-gpu-id".to_owned(),
                GpuConfig {
                    fan_control_enabled: true,
                    fan_control_settings: Some(FanControlSettings {
                        curve: FanCurve::default(),
                        temperature_key: "edge".to_owned(),
                        interval_ms: 500,
                    }),
                },
            )]
            .into(),
        };
        let data = serde_yaml::to_string(&config).unwrap();
        let deserialized_config: Config = serde_yaml::from_str(&data).unwrap();
        assert_eq!(config, deserialized_config);
    }
}