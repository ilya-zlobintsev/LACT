use crate::FanCurveMap;
use amdgpu_sysfs::gpu_handle::PerformanceLevel;
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
    SetPowerProfileMode {
        id: &'a str,
        index: Option<u16>,
    },
    EnableOverdrive,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum SetClocksCommand {
    MaxCoreClock(u32),
    MaxMemoryClock(u32),
    MaxVoltage(u32),
    MinCoreClock(u32),
    MinMemoryClock(u32),
    MinVoltage(u32),
    VoltageOffset(i32),
    Reset,
}
