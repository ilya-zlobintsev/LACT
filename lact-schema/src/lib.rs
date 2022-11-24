mod request;
mod response;

#[cfg(test)]
mod tests;

pub use request::Request;
pub use response::Response;

pub use amdgpu_sysfs::{
    gpu_handle::{
        overdrive::{ClocksTable, ClocksTableGen},
        PerformanceLevel,
    },
    hw_mon::Temperature,
};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
};

pub type FanCurveMap = BTreeMap<i32, f32>;

#[derive(Serialize, Deserialize, Debug)]
pub struct Pong<'a> {
    pub version: &'a str,
    pub profile: &'a str,
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
    pub clocks_table: Option<ClocksTableGen>,
    pub clocks_info: ClocksInfo,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, Copy)]
pub struct ClocksInfo {
    pub max_sclk: Option<u32>,
    pub max_mclk: Option<u32>,
}

impl<T: ClocksTable> From<&T> for ClocksInfo {
    fn from(table: &T) -> Self {
        let max_sclk = table.get_max_sclk();
        let max_mclk = table.get_max_mclk();
        Self { max_sclk, max_mclk }
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
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FanStats {
    pub control_enabled: bool,
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
