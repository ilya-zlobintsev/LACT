use std::fmt;

use crate::{
    FanOptions, ProfileRule,
    config::{GpuConfig, Profile, ProfileHooks},
};
use amdgpu_sysfs::gpu_handle::{PerformanceLevel, PowerLevelKind};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
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
#[serde(tag = "command", rename_all = "snake_case")]
pub enum ConfirmCommand {
    Confirm,
    Revert,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
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
#[serde(rename_all = "snake_case")]
pub enum ClockspeedType {
    MaxCoreClock,
    MinCoreClock,
    GpuClockOffset(u32),

    MinVoltage,
    MaxVoltage,
    VoltageOffset,

    MaxMemoryClock,
    MinMemoryClock,
    MemClockOffset(u32),

    GpuVfCurveClock(u8),
    GpuVfCurveVoltage(u8),

    MemVfCurveClock(u8),
    MemVfCurveVoltage(u8),

    Reset,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
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

#[cfg(test)]
mod tests {
    use crate::{
        Request,
        request::{ClockspeedType, SetClocksCommand},
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
