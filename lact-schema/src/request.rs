#![allow(clippy::needless_lifetimes)]

use std::fmt;

use crate::{
    config::{GpuConfig, Profile, ProfileHooks},
    FanOptions, ProfileRule,
};
use amdgpu_sysfs::gpu_handle::{PerformanceLevel, PowerLevelKind};
#[cfg(feature = "schema")]
use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(tag = "command", content = "args", rename_all = "snake_case")]
pub enum Request<'a> {
    Ping,
    ListDevices,
    SystemInfo,
    DeviceInfo {
        id: &'a str,
    },
    DeviceStats {
        id: &'a str,
    },
    DeviceClocksInfo {
        id: &'a str,
    },
    DevicePowerProfileModes {
        id: &'a str,
    },
    SetFanControl(FanOptions<'a>),
    ResetPmfw {
        id: &'a str,
    },
    SetPowerCap {
        id: &'a str,
        cap: Option<f64>,
    },
    SetPerformanceLevel {
        id: &'a str,
        #[cfg_attr(feature = "schema", schemars(schema_with = "performance_level_schema"))]
        performance_level: PerformanceLevel,
    },
    SetClocksValue {
        id: &'a str,
        command: SetClocksCommand,
    },
    BatchSetClocksValue {
        id: &'a str,
        commands: Vec<SetClocksCommand>,
    },
    SetPowerProfileMode {
        id: &'a str,
        index: Option<u16>,
        #[serde(default)]
        custom_heuristics: Vec<Vec<Option<i32>>>,
    },
    GetPowerStates {
        id: &'a str,
    },
    SetEnabledPowerStates {
        id: &'a str,
        #[cfg_attr(feature = "schema", schemars(schema_with = "power_level_kind_schema"))]
        kind: PowerLevelKind,
        states: Vec<u8>,
    },
    VbiosDump {
        id: &'a str,
    },
    ListProfiles {
        #[serde(default)]
        include_state: bool,
    },
    GetProfile {
        name: Option<String>,
    },
    SetProfile {
        name: Option<String>,
        #[serde(default)]
        auto_switch: bool,
    },
    CreateProfile {
        name: String,
        base: ProfileBase,
    },
    DeleteProfile {
        name: String,
    },
    MoveProfile {
        name: String,
        new_position: usize,
    },
    EvaluateProfileRule {
        rule: ProfileRule,
    },
    SetProfileRule {
        name: String,
        rule: Option<ProfileRule>,
        #[serde(default)]
        hooks: ProfileHooks,
    },
    GetGpuConfig {
        id: &'a str,
    },
    SetGpuConfig {
        id: &'a str,
        config: Box<GpuConfig>,
    },
    ProcessList {
        id: &'a str,
    },
    EnableOverdrive,
    DisableOverdrive,
    GenerateSnapshot,
    ConfirmPendingConfig(ConfirmCommand),
    RestConfig,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum ConfirmCommand {
    Confirm,
    Revert,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct SetClocksCommand {
    pub r#type: ClockspeedType,
    pub value: Option<i32>,
}

impl SetClocksCommand {
    pub fn reset() -> Self {
        Self {
            r#type: ClockspeedType::Reset,
            value: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy, Eq, Hash)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum ClockspeedType {
    MaxCoreClock,
    MaxMemoryClock,
    MaxVoltage,
    MinCoreClock,
    MinMemoryClock,
    MinVoltage,
    VoltageOffset,
    GpuClockOffset(u32),
    MemClockOffset(u32),
    GpuVfCurveClock(u8),
    MemVfCurveClock(u8),
    GpuVfCurveVoltage(u8),
    MemVfCurveVoltage(u8),
    Reset,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum ProfileBase {
    Empty,
    Default,
    Profile(String),
    Provided(Profile),
}

impl fmt::Display for ProfileBase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            ProfileBase::Empty => "Empty",
            ProfileBase::Default => "Default",
            ProfileBase::Profile(name) => name,
            ProfileBase::Provided(_) => "<Provided>",
        };
        text.fmt(f)
    }
}

#[cfg(feature = "schema")]
fn performance_level_schema(_gen: &mut SchemaGenerator) -> Schema {
    let enum_values = [
        PerformanceLevel::Auto,
        PerformanceLevel::High,
        PerformanceLevel::Low,
        PerformanceLevel::Manual,
    ]
    .map(|value| serde_json::to_value(value).unwrap());

    json_schema!({
        "enum": enum_values,
        "type": "string"
    })
}

#[cfg(feature = "schema")]
fn power_level_kind_schema(_gen: &mut SchemaGenerator) -> Schema {
    let enum_values = [
        PowerLevelKind::CoreClock,
        PowerLevelKind::MemoryClock,
        PowerLevelKind::SOCClock,
        PowerLevelKind::FabricClock,
        PowerLevelKind::DCEFClock,
        PowerLevelKind::PcieSpeed,
    ];

    json_schema!({
        "enum": enum_values,
        "type": "string"
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        request::{ClockspeedType, SetClocksCommand},
        Request,
    };

    #[test]
    fn deserialize_requests() {
        assert_eq!(
            Request::SetClocksValue {
                id: "asd",
                command: SetClocksCommand {
                    r#type: ClockspeedType::MaxCoreClock,
                    value: Some(2000)
                }
            },
            serde_json::from_str(r#"{"command": "set_clocks_value", "args": {"id": "asd", "command": {"type": "max_core_clock", "value": 2000}}}"#)
                .unwrap()
        );
    }
}
