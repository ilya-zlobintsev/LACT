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
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub gpu_clock_offsets: IndexMap<u32, i32>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub mem_clock_offsets: IndexMap<u32, i32>,
    pub voltage_offset: Option<i32>,
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

pub fn default_fan_static_speed() -> f32 {
    0.5
}
