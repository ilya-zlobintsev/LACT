#[cfg(feature = "args")]
pub mod args;
pub mod request;
mod response;

#[cfg(test)]
mod tests;

pub use amdgpu_sysfs;
pub use request::Request;
pub use response::Response;

use amdgpu_sysfs::{
    gpu_handle::{
        overdrive::{ClocksTable, ClocksTableGen},
        PerformanceLevel, PowerLevels,
    },
    hw_mon::Temperature,
};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
    str::FromStr,
};

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FanControlMode {
    Static,
    #[default]
    Curve,
}

impl FromStr for FanControlMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "curve" => Ok(Self::Curve),
            "static" => Ok(Self::Static),
            _ => Err("unknown fan control mode".to_string()),
        }
    }
}

pub type FanCurveMap = BTreeMap<i32, f32>;

pub fn default_fan_curve() -> FanCurveMap {
    [
        (30, 0.0),
        (40, 0.2),
        (50, 0.35),
        (60, 0.5),
        (70, 0.75),
        (80, 1.0),
    ]
    .into()
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Pong;

#[derive(Serialize, Deserialize, Debug)]
pub struct SystemInfo<'a> {
    pub version: &'a str,
    pub profile: &'a str,
    pub kernel_version: String,
    pub amdgpu_overdrive_enabled: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeviceListEntry<'a> {
    pub id: &'a str,
    pub name: Option<&'a str>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GpuPciInfo {
    pub device_pci_info: PciInfo,
    pub subsystem_pci_info: PciInfo,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeviceInfo<'a> {
    #[serde(borrow)]
    pub pci_info: Option<Cow<'a, GpuPciInfo>>,
    pub vulkan_info: Option<VulkanInfo>,
    pub driver: &'a str,
    pub vbios_version: Option<String>,
    pub link_info: LinkInfo,
    pub drm_info: Option<DrmInfo>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DrmInfo {
    pub family_name: String,
    pub asic_name: String,
    pub chip_class: String,
    pub compute_units: u32,
    pub vram_type: String,
    pub vram_bit_width: u32,
    pub vram_max_bw: String,
    pub l2_cache: u32,
    pub memory_info: Option<DrmMemoryInfo>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DrmMemoryInfo {
    pub cpu_accessible_used: u64,
    pub cpu_accessible_total: u64,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct ClocksInfo {
    pub max_sclk: Option<i32>,
    pub max_mclk: Option<i32>,
    pub max_voltage: Option<i32>,
    pub table: Option<ClocksTableGen>,
}

impl From<ClocksTableGen> for ClocksInfo {
    fn from(table: ClocksTableGen) -> Self {
        let max_sclk = table.get_max_sclk();
        let max_mclk = table.get_max_mclk();
        let max_voltage = table.get_max_sclk_voltage();
        Self {
            max_sclk,
            max_mclk,
            max_voltage,
            table: Some(table),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LinkInfo {
    pub current_width: Option<String>,
    pub current_speed: Option<String>,
    pub max_width: Option<String>,
    pub max_speed: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VulkanInfo {
    pub device_name: String,
    pub api_version: String,
    pub driver: VulkanDriverInfo,
    pub enabled_layers: Vec<String>,
    pub features: IndexMap<Cow<'static, str>, bool>,
    pub extensions: IndexMap<Cow<'static, str>, bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VulkanDriverInfo {
    pub version: u32,
    pub name: Option<String>,
    pub info: Option<String>,
    pub driver_version: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PciInfo {
    pub vendor_id: String,
    pub vendor: Option<String>,
    pub model_id: String,
    pub model: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeviceStats {
    pub fan: FanStats,
    pub clockspeed: ClockspeedStats,
    pub voltage: VoltageStats,
    pub vram: VramStats,
    pub power: PowerStats,
    pub temps: HashMap<String, Temperature>,
    pub busy_percent: Option<u8>,
    pub performance_level: Option<PerformanceLevel>,
    pub core_clock_levels: Option<PowerLevels<u64>>,
    pub memory_clock_levels: Option<PowerLevels<u64>>,
    pub pcie_clock_levels: Option<PowerLevels<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FanStats {
    pub control_enabled: bool,
    pub control_mode: Option<FanControlMode>,
    pub static_speed: Option<f64>,
    pub curve: Option<FanCurveMap>,
    pub speed_current: Option<u32>,
    pub speed_max: Option<u32>,
    pub speed_min: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct ClockspeedStats {
    pub gpu_clockspeed: Option<u64>,
    pub vram_clockspeed: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct VoltageStats {
    pub gpu: Option<u64>,
    pub northbridge: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct VramStats {
    pub total: Option<u64>,
    pub used: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct PowerStats {
    pub average: Option<f64>,
    pub cap_current: Option<f64>,
    pub cap_max: Option<f64>,
    pub cap_min: Option<f64>,
    pub cap_default: Option<f64>,
}
