use crate::server::gpu_controller::fan_control::FanCurve;
use amdgpu_sysfs::gpu_handle::{PerformanceLevel, PowerLevelKind};
use anyhow::Context;
use lact_schema::{default_fan_curve, request::SetClocksCommand, FanControlMode, PmfwOptions};
use nix::unistd::getuid;
use notify::{RecommendedWatcher, Watcher};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::{
    collections::HashMap,
    env, fs,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::sync::mpsc;
use tracing::{debug, error};

const FILE_NAME: &str = "config.yaml";
const DEFAULT_ADMIN_GROUPS: [&str; 2] = ["wheel", "sudo"];

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Config {
    pub daemon: Daemon,
    #[serde(default = "default_apply_settings_timer")]
    pub apply_settings_timer: u64,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    gpus: HashMap<String, Gpu>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    profiles: HashMap<String, Profile>,
    #[serde(default)]
    pub current_profile: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            daemon: Daemon::default(),
            apply_settings_timer: default_apply_settings_timer(),
            gpus: HashMap::new(),
            profiles: HashMap::new(),
            current_profile: None,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Profile {
    #[serde(default)]
    pub gpus: HashMap<String, Gpu>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Gpu {
    pub fan_control_enabled: bool,
    pub fan_control_settings: Option<FanControlSettings>,
    #[serde(default)]
    pub pmfw_options: PmfwOptions,
    pub power_cap: Option<f64>,
    pub performance_level: Option<PerformanceLevel>,
    #[serde(default, flatten)]
    pub clocks_configuration: ClocksConfiguration,
    pub power_profile_mode_index: Option<u16>,
    #[serde(default)]
    pub power_states: HashMap<PowerLevelKind, Vec<u8>>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ClocksConfiguration {
    pub min_core_clock: Option<i32>,
    pub min_memory_clock: Option<i32>,
    pub min_voltage: Option<i32>,
    pub max_core_clock: Option<i32>,
    pub max_memory_clock: Option<i32>,
    pub max_voltage: Option<i32>,
    pub voltage_offset: Option<i32>,
}

impl Gpu {
    pub fn is_core_clocks_used(&self) -> bool {
        self.clocks_configuration != ClocksConfiguration::default()
    }

    pub fn apply_clocks_command(&mut self, command: &SetClocksCommand) {
        let clocks = &mut self.clocks_configuration;
        match command {
            SetClocksCommand::MaxCoreClock(clock) => clocks.max_core_clock = Some(*clock),
            SetClocksCommand::MaxMemoryClock(clock) => clocks.max_memory_clock = Some(*clock),
            SetClocksCommand::MaxVoltage(voltage) => clocks.max_voltage = Some(*voltage),
            SetClocksCommand::MinCoreClock(clock) => clocks.min_core_clock = Some(*clock),
            SetClocksCommand::MinMemoryClock(clock) => clocks.min_memory_clock = Some(*clock),
            SetClocksCommand::MinVoltage(voltage) => clocks.min_voltage = Some(*voltage),
            SetClocksCommand::VoltageOffset(offset) => clocks.voltage_offset = Some(*offset),
            SetClocksCommand::Reset => {
                *clocks = ClocksConfiguration::default();
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
    pub spindown_delay_ms: Option<u64>,
    pub change_threshold: Option<u64>,
}

impl Default for FanControlSettings {
    fn default() -> Self {
        Self {
            mode: FanControlMode::default(),
            static_speed: default_fan_static_speed(),
            temperature_key: "edge".to_owned(),
            interval_ms: 500,
            curve: FanCurve(default_fan_curve()),
            spindown_delay_ms: None,
            change_threshold: None,
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

    /// Gets the GPU configs according to the current profile. Returns an error if the current profile could not be found.
    pub fn gpus(&self) -> anyhow::Result<&HashMap<String, Gpu>> {
        match &self.current_profile {
            Some(profile) => {
                let profile = self
                    .profiles
                    .get(profile)
                    .with_context(|| format!("Could not find profile '{profile}'"))?;
                Ok(&profile.gpus)
            }
            None => Ok(&self.gpus),
        }
    }

    /// Same as [`gpus`], but with a mutable reference
    pub fn gpus_mut(&mut self) -> anyhow::Result<&mut HashMap<String, Gpu>> {
        match &self.current_profile {
            Some(profile) => {
                let profile = self
                    .profiles
                    .get_mut(profile)
                    .with_context(|| format!("Could not find profile '{profile}'"))?;
                Ok(&mut profile.gpus)
            }
            None => Ok(&mut self.gpus),
        }
    }
}

pub fn start_watcher(config_last_applied: Arc<Mutex<Instant>>) -> mpsc::UnboundedReceiver<Config> {
    let (config_tx, config_rx) = mpsc::unbounded_channel();
    let (event_tx, event_rx) = std::sync::mpsc::channel();

    tokio::task::spawn_blocking(move || {
        let mut watcher = RecommendedWatcher::new(event_tx, notify::Config::default())
            .expect("Could not create config file watcher");

        let config_path = get_path();
        let watch_path = config_path
            .parent()
            .expect("Config path always has a parent");
        watcher
            .watch(watch_path, notify::RecursiveMode::Recursive)
            .expect("Could not subscribe to config file changes");

        for res in event_rx {
            debug!("got config file event {res:?}");
            match res {
                Ok(event) => {
                    use notify::EventKind;

                    let elapsed = config_last_applied.lock().unwrap().elapsed();
                    if elapsed < Duration::from_millis(50) {
                        debug!("config was applied very recently, skipping fs event");
                        continue;
                    }

                    if !event.paths.contains(&config_path) {
                        continue;
                    }

                    match event.kind {
                        EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_) => {
                            match Config::load() {
                                Ok(Some(new_config)) => config_tx.send(new_config).unwrap(),
                                Ok(None) => error!("config was removed!"),
                                Err(err) => {
                                    error!("could not read config after it was changed: {err:#}");
                                }
                            }
                        }
                        _ => (),
                    }
                }
                Err(err) => error!("filesystem event error: {err}"),
            }
        }

        debug!("registered config file event listener at path {watch_path:?}");
    });

    config_rx
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
    use super::{ClocksConfiguration, Config, Daemon, FanControlSettings, Gpu};
    use crate::server::gpu_controller::fan_control::FanCurve;
    use lact_schema::{FanControlMode, PmfwOptions};
    use std::collections::HashMap;

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
                        spindown_delay_ms: Some(5000),
                        change_threshold: Some(3),
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

    #[test]
    fn clocks_configuration_applied() {
        let mut gpu = Gpu {
            fan_control_enabled: false,
            fan_control_settings: None,
            pmfw_options: PmfwOptions::default(),
            power_cap: None,
            performance_level: None,
            clocks_configuration: ClocksConfiguration::default(),
            power_profile_mode_index: None,
            power_states: HashMap::new(),
        };

        assert!(!gpu.is_core_clocks_used());
        gpu.clocks_configuration.voltage_offset = Some(10);
        assert!(gpu.is_core_clocks_used());
    }
}
