use amdgpu_sysfs::gpu_handle::{PerformanceLevel, PowerLevelKind};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::{
    default_fan_curve,
    request::{ClockspeedType, SetClocksCommand},
    FanControlMode, FanCurveMap, PmfwOptions,
};

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct GpuConfig {
    #[serde(default)]
    pub fan_control_enabled: bool,
    pub fan_control_settings: Option<FanControlSettings>,
    #[serde(default, skip_serializing_if = "PmfwOptions::is_empty")]
    pub pmfw_options: PmfwOptions,
    pub power_cap: Option<f64>,
    pub performance_level: Option<PerformanceLevel>,
    #[serde(default, flatten)]
    pub clocks_configuration: ClocksConfiguration,
    pub power_profile_mode_index: Option<u16>,
    /// Outer vector is for power profile components, inner vector is for the heuristics within a component
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub custom_power_profile_mode_hueristics: Vec<Vec<Option<i32>>>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub power_states: IndexMap<PowerLevelKind, Vec<u8>>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ClocksConfiguration {
    pub min_core_clock: Option<i32>,
    pub min_memory_clock: Option<i32>,
    pub min_voltage: Option<i32>,
    pub max_core_clock: Option<i32>,
    pub max_memory_clock: Option<i32>,
    pub max_voltage: Option<i32>,
    #[serde(
        default,
        skip_serializing_if = "IndexMap::is_empty",
        deserialize_with = "offsets::deserialize"
    )]
    pub gpu_clock_offsets: IndexMap<u32, i32>,
    #[serde(
        default,
        skip_serializing_if = "IndexMap::is_empty",
        deserialize_with = "offsets::deserialize"
    )]
    pub mem_clock_offsets: IndexMap<u32, i32>,
    pub voltage_offset: Option<i32>,
}

mod offsets {
    use indexmap::IndexMap;
    use serde::{de::Error, Deserialize, Deserializer};
    use serde_json::Value;

    pub fn deserialize<'a, D: Deserializer<'a>>(
        deserializer: D,
    ) -> Result<IndexMap<u32, i32>, D::Error> {
        let map: IndexMap<Value, i32> = IndexMap::deserialize(deserializer)?;

        map.into_iter()
            .map(|(key, value)| {
                let parsed_key = match &key {
                    Value::Number(number) => {
                        number.as_i64().and_then(|val| u32::try_from(val).ok())
                    }
                    Value::String(s) => s.parse::<u32>().ok(),
                    _ => None,
                };
                let key =
                    parsed_key.ok_or_else(|| D::Error::custom(format!("Invalid key {key}")))?;

                Ok((key, value))
            })
            .collect()
    }
}

impl GpuConfig {
    pub fn is_core_clocks_used(&self) -> bool {
        self.clocks_configuration != ClocksConfiguration::default()
    }

    pub fn apply_clocks_command(&mut self, command: &SetClocksCommand) {
        let clocks = &mut self.clocks_configuration;
        let value = command.value;
        match command.r#type {
            ClockspeedType::MaxCoreClock => clocks.max_core_clock = value,
            ClockspeedType::MaxMemoryClock => clocks.max_memory_clock = value,
            ClockspeedType::MaxVoltage => clocks.max_voltage = value,
            ClockspeedType::MinCoreClock => clocks.min_core_clock = value,
            ClockspeedType::MinMemoryClock => clocks.min_memory_clock = value,
            ClockspeedType::MinVoltage => clocks.min_voltage = value,
            ClockspeedType::VoltageOffset => clocks.voltage_offset = value,
            ClockspeedType::GpuClockOffset(pstate) => match value {
                Some(value) => {
                    clocks.gpu_clock_offsets.insert(pstate, value);
                }
                None => {
                    clocks.gpu_clock_offsets.shift_remove(&pstate);
                }
            },
            ClockspeedType::MemClockOffset(pstate) => match value {
                Some(value) => {
                    clocks.mem_clock_offsets.insert(pstate, value);
                }
                None => {
                    clocks.mem_clock_offsets.shift_remove(&pstate);
                }
            },
            ClockspeedType::Reset => {
                *clocks = ClocksConfiguration::default();
                assert!(!self.is_core_clocks_used());
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FanCurve(pub FanCurveMap);

impl Default for FanCurve {
    fn default() -> Self {
        Self(default_fan_curve())
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FanControlSettings {
    #[serde(default)]
    pub mode: FanControlMode,
    #[serde(default = "default_fan_static_speed")]
    pub static_speed: f32,
    pub temperature_key: String,
    pub interval_ms: u64,
    pub curve: FanCurve,
    pub spindown_delay_ms: Option<u64>,
    pub change_threshold: Option<u64>,
    pub auto_threshold: Option<u64>,
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
            auto_threshold: None,
        }
    }
}

pub fn default_fan_static_speed() -> f32 {
    0.5
}

#[cfg(test)]
mod tests {
    use super::GpuConfig;

    #[test]
    fn deserialize_config_json() {
        let data = r#"{"fan_control_enabled":false,"fan_control_settings":{"mode":"curve","static_speed":0.5938412,"temperature_key":"edge","interval_ms":500,"curve":{"40":0.3,"50":0.35,"60":0.5,"70":0.75,"80":1.0},"spindown_delay_ms":1000,"change_threshold":2},"power_cap":318.0,"gpu_clock_offsets":{"0":-64}}"#;
        let config: GpuConfig = serde_json::from_str(data).unwrap();
        assert_eq!(
            -64,
            *config
                .clocks_configuration
                .gpu_clock_offsets
                .get(&0)
                .unwrap()
        );
    }
}
