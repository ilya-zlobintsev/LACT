pub mod request;
pub mod response;
#[cfg(test)]
mod tests;

pub use amdgpu_sysfs::{gpu_handle::PerformanceLevel, hw_mon::Temperature};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::HashMap};

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
    pub driver_name: Option<String>,
    pub supported_features: IndexMap<Cow<'static, str>, bool>,
    pub supported_extensions: IndexMap<Cow<'static, str>, bool>,
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
    pub fan_speed_current: Option<u32>,
    pub fan_speed_max: Option<u32>,
    pub fan_speed_min: Option<u32>,
    pub fan_control_enabled: bool,
    pub temps: HashMap<String, Temperature>,
    pub total_vram: Option<u64>,
    pub used_vram: Option<u64>,
    pub busy_percent: Option<u8>,
    pub performance_level: Option<PerformanceLevel>,
}
