use crate::server::gpu_controller::fan_control::FanCurve;
use anyhow::Context;
use lact_schema::{
    amdgpu_sysfs::gpu_handle::{PerformanceLevel, PowerLevelKind},
    default_fan_curve,
    request::SetClocksCommand,
    FanControlMode,
};
use nix::unistd::getuid;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::{collections::HashMap, env, fs, path::PathBuf};
use tracing::debug;

const FILE_NAME: &str = "config.yaml";
const DEFAULT_ADMIN_GROUPS: [&str; 2] = ["wheel", "sudo"];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Config {
    pub daemon: Daemon,
    #[serde(default = "default_apply_settings_timer")]
    pub apply_settings_timer: u64,
    pub gpus: HashMap<String, Gpu>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            daemon: Daemon::default(),
            apply_settings_timer: default_apply_settings_timer(),
            gpus: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Daemon {
    pub log_level: String,
    pub admin_groups: Vec<String>,
    #[serde(default)]
    pub disable_clocks_cleanup: bool,
}

impl Default for Daemon {
    fn default() -> Self {
        Self {
            log_level: "info".to_owned(),
            admin_groups: DEFAULT_ADMIN_GROUPS.map(str::to_owned).to_vec(),
            disable_clocks_cleanup: false,
        }
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Gpu {
    pub fan_control_enabled: bool,
    pub fan_control_settings: Option<FanControlSettings>,
    pub power_cap: Option<f64>,
    pub performance_level: Option<PerformanceLevel>,
    pub min_core_clock: Option<i32>,
    pub min_memory_clock: Option<i32>,
    pub min_voltage: Option<i32>,
    pub max_core_clock: Option<i32>,
    pub max_memory_clock: Option<i32>,
    pub max_voltage: Option<i32>,
    pub voltage_offset: Option<i32>,
    pub power_profile_mode_index: Option<u16>,
    pub power_states: HashMap<PowerLevelKind, Vec<u8>>,
}

impl Gpu {
    pub fn is_core_clocks_used(&self) -> bool {
        [
            self.min_core_clock,
            self.min_memory_clock,
            self.max_voltage,
            self.max_core_clock,
            self.max_memory_clock,
            self.min_voltage,
        ]
        .iter()
        .any(Option::is_some)
    }

    pub fn apply_clocks_command(&mut self, command: &SetClocksCommand) {
        match command {
            SetClocksCommand::MaxCoreClock(clock) => self.max_core_clock = Some(*clock),
            SetClocksCommand::MaxMemoryClock(clock) => self.max_memory_clock = Some(*clock),
            SetClocksCommand::MaxVoltage(voltage) => self.max_voltage = Some(*voltage),
            SetClocksCommand::MinCoreClock(clock) => self.min_core_clock = Some(*clock),
            SetClocksCommand::MinMemoryClock(clock) => self.min_memory_clock = Some(*clock),
            SetClocksCommand::MinVoltage(voltage) => self.min_voltage = Some(*voltage),
            SetClocksCommand::VoltageOffset(offset) => self.voltage_offset = Some(*offset),
            SetClocksCommand::Reset => {
                self.min_core_clock = None;
                self.min_memory_clock = None;
                self.min_voltage = None;
                self.max_core_clock = None;
                self.max_memory_clock = None;
                self.max_voltage = None;

                assert!(!self.is_core_clocks_used());
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FanControlSettings {
    #[serde(default)]
    pub mode: FanControlMode,
    #[serde(default = "default_fan_static_speed")]
    pub static_speed: f64,
    pub temperature_key: String,
    pub interval_ms: u64,
    pub curve: FanCurve,
}

impl Default for FanControlSettings {
    fn default() -> Self {
        Self {
            mode: FanControlMode::default(),
            static_speed: default_fan_static_speed(),
            temperature_key: "edge".to_owned(),
            interval_ms: 500,
            curve: FanCurve(default_fan_curve()),
        }
    }
}

pub fn default_fan_static_speed() -> f64 {
    0.5
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
        debug!("saving config to {path:?}");
        let raw_config = serde_yaml::to_string(self)?;
        fs::write(path, raw_config).context("Could not write config")
    }

    pub fn load_or_create() -> anyhow::Result<Self> {
        if let Some(config) = Config::load()? {
            Ok(config)
        } else {
            let config = Config::default();
            config.save()?;
            Ok(config)
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

fn default_apply_settings_timer() -> u64 {
    5
}

#[cfg(test)]
mod tests {
    use lact_schema::FanControlMode;

    use super::{Config, Daemon, FanControlSettings, Gpu};
    use crate::server::gpu_controller::fan_control::FanCurve;

    #[test]
    fn serde_de_full() {
        let config = Config {
            daemon: Daemon::default(),
            gpus: [(
                "my-gpu-id".to_owned(),
                Gpu {
                    fan_control_enabled: true,
                    fan_control_settings: Some(FanControlSettings {
                        curve: FanCurve::default(),
                        temperature_key: "edge".to_owned(),
                        interval_ms: 500,
                        mode: FanControlMode::Curve,
                        static_speed: 0.5,
                    }),
                    ..Default::default()
                },
            )]
            .into(),
            ..Default::default()
        };
        let data = serde_yaml::to_string(&config).unwrap();
        let deserialized_config: Config = serde_yaml::from_str(&data).unwrap();
        assert_eq!(config, deserialized_config);
    }
}
