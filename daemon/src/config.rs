use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, io, path::PathBuf};

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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub fan_control_enabled: bool,
    pub fan_curve: BTreeMap<i32, f64>,
}

impl Config {
    pub fn new() -> Self {
        let mut fan_curve: BTreeMap<i32, f64> = BTreeMap::new();
        fan_curve.insert(20, 0f64);
        fan_curve.insert(40, 0f64);
        fan_curve.insert(60, 50f64);
        fan_curve.insert(80, 80f64);
        fan_curve.insert(100, 100f64);

        Config {
            fan_curve,
            fan_control_enabled: false,
        }
    }

    pub fn read_from_file(path: &PathBuf) -> Result<Self, ConfigError> {
        let json = fs::read_to_string(path)?;

        Ok(serde_json::from_str::<Config>(&json)?)
    }

    pub fn save(&self, path: &PathBuf) -> Result<(), ConfigError> {
        let json = serde_json::json!(self);

        Ok(fs::write(path, &json.to_string())?)
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
