use std::fmt;

use crate::{FanOptions, ProfileRule};
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
    MaxMemoryClock,
    MaxVoltage,
    MinCoreClock,
    MinMemoryClock,
    MinVoltage,
    VoltageOffset,
    Reset,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ProfileBase {
    Empty,
    Default,
    Profile(String),
}

impl fmt::Display for ProfileBase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            ProfileBase::Empty => "Empty",
            ProfileBase::Default => "Default",
            ProfileBase::Profile(name) => name,
        };
        text.fmt(f)
    }
}
