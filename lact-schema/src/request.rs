use crate::{FanControlMode, FanCurveMap};
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
    SetFanControl {
        id: &'a str,
        enabled: bool,
        mode: Option<FanControlMode>,
        static_speed: Option<f64>,
        curve: Option<FanCurveMap>,
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
    },
    GetPowerStates {
        id: &'a str,
    },
    SetEnabledPowerStates {
        id: &'a str,
        kind: PowerLevelKind,
        states: Vec<u8>,
    },
    EnableOverdrive,
    GenerateSnapshot,
    ConfirmPendingConfig(ConfirmCommand),
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum ConfirmCommand {
    Confirm,
    Revert,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum SetClocksCommand {
    MaxCoreClock(i32),
    MaxMemoryClock(i32),
    MaxVoltage(i32),
    MinCoreClock(i32),
    MinMemoryClock(i32),
    MinVoltage(i32),
    VoltageOffset(i32),
    Reset,
}
