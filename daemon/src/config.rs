use serde::{Deserialize, Serialize};
use std::{collections::{BTreeMap, HashMap}, fs, io, path::PathBuf};

use crate::gpu_controller::PowerProfile;

#[derive(Debug)]
pub enum ConfigError {
    IoError(io::Error),
    ParseError(serde_json::Error),
}

impl From<io::Error> for ConfigError {
    fn from(error: io::Error) -> Self {
        ConfigError::IoError(error)
    }
}

impl From<serde_json::Error> for ConfigError {
    fn from(error: serde_json::Error) -> Self {
        ConfigError::ParseError(error)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Hash, Eq)]
pub struct GpuIdentifier {
    pub pci_id: String,
    pub card_model: String,
    pub gpu_model: String,
    pub path: PathBuf,
}

impl PartialEq for GpuIdentifier {
    fn eq(&self, other: &Self) -> bool {
        self.pci_id == other.pci_id && self.gpu_model == other.gpu_model && self.card_model == other.card_model
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GpuConfig {
    pub fan_control_enabled: bool,
    pub fan_curve: BTreeMap<i32, f64>,
    pub power_cap: i32,
    pub power_profile: PowerProfile,
    pub gpu_power_states: BTreeMap<u32, (i32, i32)>,  //<id, (clockspeed, voltage)>
    pub vram_power_states: BTreeMap<u32, (i32, i32)>,
}

impl GpuConfig {
    pub fn new() -> Self {
        let mut fan_curve: BTreeMap<i32, f64> = BTreeMap::new();
        fan_curve.insert(20, 0f64);
        fan_curve.insert(40, 0f64);
        fan_curve.insert(60, 50f64);
        fan_curve.insert(80, 80f64);
        fan_curve.insert(100, 100f64);

        GpuConfig {
            fan_curve,
            fan_control_enabled: false,
            power_cap: -1,
            power_profile: PowerProfile::Auto,
            gpu_power_states: BTreeMap::new(),
            vram_power_states: BTreeMap::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub gpu_configs: HashMap<u32, (GpuIdentifier, GpuConfig)>,
    pub config_path: PathBuf,
}

impl Config {
    pub fn new(config_path: &PathBuf) -> Self {
        let gpu_configs: HashMap<u32, (GpuIdentifier, GpuConfig)> = HashMap::new();

        Config {
            gpu_configs,
            config_path: config_path.clone(),
        }
    }

    pub fn read_from_file(path: &PathBuf) -> Result<Self, ConfigError> {
        let json = fs::read_to_string(path)?;

        Ok(serde_json::from_str::<Config>(&json)?)
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        let json = serde_json::json!(self);
        log::info!("saving {}", json.to_string());

        Ok(fs::write(&self.config_path, &json.to_string())?)
    }
}

/*#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_config() -> Result<(), ConfigError> {
        let c = Config::new();
        c.save(PathBuf::from("/tmp/config.json"))
    }
}*/
