#[cfg(feature = "args")]
pub mod args;
pub mod request;
mod response;

#[cfg(test)]
mod tests;

pub use request::Request;
pub use response::Response;

use amdgpu_sysfs::{
    gpu_handle::{
        fan_control::FanInfo,
        overdrive::{ClocksTable, ClocksTableGen},
        PerformanceLevel,
    },
    hw_mon::Temperature,
};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
    fmt,
    str::FromStr,
};

pub const GIT_COMMIT: &str = env!("VERGEN_GIT_SHA");

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
    [(40, 0.2), (50, 0.35), (60, 0.5), (70, 0.75), (80, 1.0)].into()
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Pong;

#[derive(Serialize, Deserialize, Debug)]
pub struct SystemInfo {
    pub version: String,
    pub commit: Option<String>,
    pub profile: String,
    pub kernel_version: String,
    pub amdgpu_overdrive_enabled: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeviceListEntry {
    pub id: String,
    pub name: Option<String>,
}

impl fmt::Display for DeviceListEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.name {
            Some(name) => name.fmt(f),
            None => self.id.fmt(f),
        }
    }
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
    pub driver: String,
    pub vbios_version: Option<String>,
    pub link_info: LinkInfo,
    pub drm_info: Option<DrmInfo>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DrmInfo {
    pub device_name: Option<String>,
    pub pci_revision_id: Option<u32>,
    pub family_name: Option<String>,
    pub family_id: Option<u32>,
    pub asic_name: Option<String>,
    pub chip_class: Option<String>,
    pub compute_units: Option<u32>,
    pub cuda_cores: Option<u32>,
    pub vram_type: Option<String>,
    pub vram_clock_ratio: f64,
    pub vram_bit_width: Option<u32>,
    pub vram_max_bw: Option<String>,
    pub l1_cache_per_cu: Option<u32>,
    pub l2_cache: Option<u32>,
    pub l3_cache_mb: Option<u32>,
    pub memory_info: Option<DrmMemoryInfo>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DrmMemoryInfo {
    pub cpu_accessible_used: u64,
    pub cpu_accessible_total: u64,
    pub resizeable_bar: Option<bool>,
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

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
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

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DeviceStats {
    pub fan: FanStats,
    pub clockspeed: ClockspeedStats,
    pub voltage: VoltageStats,
    pub vram: VramStats,
    pub power: PowerStats,
    pub temps: HashMap<String, Temperature>,
    pub busy_percent: Option<u8>,
    pub performance_level: Option<PerformanceLevel>,
    pub core_power_state: Option<usize>,
    pub memory_power_state: Option<usize>,
    pub pcie_power_state: Option<usize>,
    pub throttle_info: Option<BTreeMap<String, Vec<String>>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct FanStats {
    pub control_enabled: bool,
    pub control_mode: Option<FanControlMode>,
    pub static_speed: Option<f64>,
    pub curve: Option<FanCurveMap>,
    pub pwm_current: Option<u8>,
    pub speed_current: Option<u32>,
    pub speed_max: Option<u32>,
    pub speed_min: Option<u32>,
    pub spindown_delay_ms: Option<u64>,
    pub change_threshold: Option<u64>,
    // RDNA3+ params
    #[serde(default)]
    pub pmfw_info: PmfwInfo,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PmfwInfo {
    pub acoustic_limit: Option<FanInfo>,
    pub acoustic_target: Option<FanInfo>,
    pub target_temp: Option<FanInfo>,
    pub minimum_pwm: Option<FanInfo>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default)]
pub struct ClockspeedStats {
    pub gpu_clockspeed: Option<u64>,
    pub current_gfxclk: Option<u16>,
    pub vram_clockspeed: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default)]
pub struct VoltageStats {
    pub gpu: Option<u64>,
    pub northbridge: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default)]
pub struct VramStats {
    pub total: Option<u64>,
    pub used: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default)]
pub struct PowerStats {
    pub average: Option<f64>,
    pub current: Option<f64>,
    pub cap_current: Option<f64>,
    pub cap_max: Option<f64>,
    pub cap_min: Option<f64>,
    pub cap_default: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PowerStates {
    pub core: Vec<PowerState>,
    pub vram: Vec<PowerState>,
}

impl PowerStates {
    pub fn is_empty(&self) -> bool {
        self.core.is_empty() && self.vram.is_empty()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct PowerState {
    pub enabled: bool,
    pub min_value: Option<u64>,
    pub value: u64,
    pub index: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitramfsType {
    Debian,
    Mkinitcpio,
    Dracut,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PmfwOptions {
    pub acoustic_limit: Option<u32>,
    pub acoustic_target: Option<u32>,
    pub minimum_pwm: Option<u32>,
    pub target_temperature: Option<u32>,
}

impl PmfwOptions {
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct FanOptions<'a> {
    pub id: &'a str,
    pub enabled: bool,
    pub mode: Option<FanControlMode>,
    pub static_speed: Option<f64>,
    pub curve: Option<FanCurveMap>,
    #[serde(default)]
    pub pmfw: PmfwOptions,
    pub spindown_delay_ms: Option<u64>,
    pub change_threshold: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct ProfilesInfo {
    pub profiles: Vec<String>,
    pub current_profile: Option<String>,
}
