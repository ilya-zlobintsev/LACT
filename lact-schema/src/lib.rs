#[cfg(feature = "args")]
pub mod args;
pub mod config;
pub mod i18n;
mod profiles;
pub mod request;
mod response;

#[cfg(test)]
mod tests;

use i18n_embed_fl::fl;
pub use request::Request;
pub use response::Response;

use amdgpu_sysfs::{
    gpu_handle::{
        fan_control::FanInfo,
        overdrive::{ClocksTable as _, ClocksTableGen as AmdClocksTableGen},
        PerformanceLevel,
    },
    hw_mon::Temperature,
};
use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fmt::{self, Debug, Display, Write},
    str::FromStr,
    sync::Arc,
};

use crate::{config::ProfileHooks, i18n::LANGUAGE_LOADER};

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
    [(40, 0.3), (50, 0.35), (60, 0.5), (70, 0.75), (80, 1.0)].into()
}

pub fn bytes_to_mib(bytes: u64) -> f64 {
    bytes as f64 / 1024.0 / 1024.0
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Pong;

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SystemInfo {
    pub version: String,
    pub commit: Option<String>,
    pub profile: String,
    pub distro: Option<String>,
    pub kernel_version: String,
    pub amdgpu_overdrive_enabled: Option<bool>,
    pub amdgpu_params_configurator: Option<AmdgpuParamsConfigurator>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeviceListEntry {
    pub id: String,
    pub name: Option<String>,
    #[serde(default)]
    pub device_type: DeviceType,
}

impl Display for DeviceListEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.name {
            Some(name) => Display::fmt(name, f),
            None => Display::fmt(&self.id, f),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq)]
pub enum DeviceType {
    #[default]
    Dedicated,
    Integrated,
}

impl Display for DeviceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            DeviceType::Dedicated => "Dedicated",
            DeviceType::Integrated => "Integrated",
        };
        Display::fmt(s, f)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GpuPciInfo {
    pub device_pci_info: PciInfo,
    pub subsystem_pci_info: PciInfo,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum DeviceFlag {
    ConfigurableFanControl,
    DumpableVBios,
    HasPmfw,
    AutoFanThreshold,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeviceInfo {
    pub pci_info: Option<GpuPciInfo>,
    #[serde(default)]
    pub vulkan_instances: Vec<VulkanInfo>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub opencl_instances: Vec<OpenCLInfo>,
    pub driver: String,
    pub vbios_version: Option<String>,
    pub link_info: LinkInfo,
    pub drm_info: Option<DrmInfo>,
    #[serde(default)]
    pub flags: Vec<DeviceFlag>,
}

impl DeviceInfo {
    pub fn vram_clock_ratio(&self) -> f64 {
        self.drm_info
            .as_ref()
            .map(|info| info.vram_clock_ratio)
            .unwrap_or(1.0)
    }

    pub fn info_elements(&self, stats: Option<&DeviceStats>) -> Vec<(String, Option<String>)> {
        let pci_info = self.pci_info.as_ref();

        let mut gpu_model = self
            .drm_info
            .as_ref()
            .and_then(|drm| drm.device_name.as_deref())
            .or_else(|| pci_info.and_then(|pci_info| pci_info.device_pci_info.model.as_deref()))
            .unwrap_or("Unknown")
            .to_owned();

        let mut card_manufacturer = pci_info
            .and_then(|info| info.subsystem_pci_info.vendor.as_deref())
            .unwrap_or("Unknown")
            .to_owned();

        let mut card_model = pci_info
            .and_then(|info| info.subsystem_pci_info.model.as_deref())
            .unwrap_or("Unknown")
            .to_owned();

        if let Some(pci_info) = &self.pci_info {
            match self.drm_info {
                Some(DrmInfo {
                    pci_revision_id: Some(pci_rev),
                    ..
                }) => {
                    let _ = write!(
                        gpu_model,
                        " (0x{}:0x{}:0x{pci_rev:X})",
                        pci_info.device_pci_info.vendor_id, pci_info.device_pci_info.model_id,
                    );
                }
                _ => {
                    let _ = write!(
                        gpu_model,
                        " (0x{}:0x{})",
                        pci_info.device_pci_info.vendor_id, pci_info.device_pci_info.model_id
                    );
                }
            }

            let _ = write!(
                card_manufacturer,
                " (0x{})",
                pci_info.subsystem_pci_info.vendor_id
            );

            let _ = write!(card_model, " (0x{})", pci_info.subsystem_pci_info.model_id);
        };

        let mut elements = vec![
            (fl!(LANGUAGE_LOADER, "gpu-model"), Some(gpu_model)),
            (fl!(LANGUAGE_LOADER, "subvendor"), Some(card_manufacturer)),
            (fl!(LANGUAGE_LOADER, "subdevice"), Some(card_model)),
            (
                fl!(LANGUAGE_LOADER, "driver-used"),
                Some(self.driver.clone()),
            ),
            (
                fl!(LANGUAGE_LOADER, "vbios-version"),
                self.vbios_version.clone(),
            ),
        ];

        if let Some(stats) = stats {
            elements.push((
                fl!(LANGUAGE_LOADER, "vram-size"),
                stats
                    .vram
                    .total
                    .map(|size| format!("{} Mb", bytes_to_mib(size))),
            ));
        }

        let mut elements = elements
            .into_iter()
            .map(|(key, value)| (key.to_owned(), value))
            .collect::<Vec<_>>();

        if let Some(drm_info) = &self.drm_info {
            let mut vram_type = drm_info.vram_type.clone();
            if let Some(vram_type) = &mut vram_type {
                if let Some(width) = drm_info.vram_bit_width {
                    write!(vram_type, " {width}-bit").unwrap();
                }

                if let Some(vram_vendor) = &drm_info.vram_vendor {
                    write!(vram_type, " ({vram_vendor})").unwrap();
                }

                if let Some(bw) = &drm_info.vram_max_bw {
                    if bw != "0" {
                        write!(vram_type, " {bw} GiB/s").unwrap();
                    }
                }
            }

            elements.extend([
                (
                    fl!(LANGUAGE_LOADER, "gpu-family"),
                    drm_info.family_name.clone(),
                ),
                (
                    fl!(LANGUAGE_LOADER, "asic-name"),
                    drm_info.asic_name.clone(),
                ),
                (
                    fl!(LANGUAGE_LOADER, "compute-units"),
                    drm_info.compute_units.map(|count| count.to_string()),
                ),
                (
                    fl!(LANGUAGE_LOADER, "execution-units"),
                    drm_info
                        .intel
                        .execution_units
                        .map(|count| count.to_string()),
                ),
                (
                    fl!(LANGUAGE_LOADER, "subslices"),
                    drm_info.intel.subslices.map(|count| count.to_string()),
                ),
                (
                    fl!(LANGUAGE_LOADER, "cuda-cores"),
                    drm_info.cuda_cores.map(|count| count.to_string()),
                ),
                (
                    fl!(LANGUAGE_LOADER, "hardware-count", name = "SM"),
                    drm_info
                        .streaming_multiprocessors
                        .map(|count| count.to_string()),
                ),
                (
                    fl!(LANGUAGE_LOADER, "hardware-count", name = "ROP"),
                    drm_info.rop_info.as_ref().map(|rop| {
                        format!(
                            "{} ({} * {})",
                            rop.operations_count, rop.unit_count, rop.operations_factor
                        )
                    }),
                ),
                (fl!(LANGUAGE_LOADER, "isa"), drm_info.isa.clone()),
                (fl!(LANGUAGE_LOADER, "vram-type"), vram_type),
            ]);

            if let Some(memory_info) = &drm_info.memory_info {
                if let Some(rebar) = memory_info.resizeable_bar {
                    let rebar = if rebar {
                        fl!(LANGUAGE_LOADER, "enabled")
                    } else {
                        fl!(LANGUAGE_LOADER, "disabled")
                    };
                    elements.push((fl!(LANGUAGE_LOADER, "rebar"), Some(rebar.to_owned())));
                }

                elements.push((
                    fl!(LANGUAGE_LOADER, "cpu-vram"),
                    Some(format!(
                        "{} MiB",
                        memory_info.cpu_accessible_total / 1024 / 1024
                    )),
                ));
            }
        }

        if let (Some(max_link_speed), Some(max_link_width)) =
            (&self.link_info.max_speed, &self.link_info.max_width)
        {
            if let (Some(current_link_speed), Some(current_link_width)) =
                (&self.link_info.current_speed, &self.link_info.current_width)
            {
                elements.push((fl!(LANGUAGE_LOADER, "pcie-speed"), Some(format!("{current_link_speed} x{current_link_width} (Max: {max_link_speed} x{max_link_width})"))));
            }
        }

        elements
    }
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DrmInfo {
    pub device_name: Option<String>,
    pub pci_revision_id: Option<u32>,
    pub family_name: Option<String>,
    pub family_id: Option<u32>,
    pub asic_name: Option<String>,
    pub chip_class: Option<String>,
    pub compute_units: Option<u32>,
    pub isa: Option<String>,
    pub streaming_multiprocessors: Option<u32>,
    pub cuda_cores: Option<u32>,
    pub vram_type: Option<String>,
    pub vram_vendor: Option<String>,
    pub vram_clock_ratio: f64,
    pub vram_bit_width: Option<u32>,
    pub vram_max_bw: Option<String>,
    pub cache_info: Option<CacheInfo>,
    pub rop_info: Option<RopInfo>,
    pub memory_info: Option<DrmMemoryInfo>,
    #[serde(flatten)]
    pub intel: IntelDrmInfo,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CacheInfo {
    Amd(Vec<(AmdCacheInstance, u16)>),
    Nvidia { l2: u32 },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AmdCacheInstance {
    pub types: Vec<CacheType>,
    pub level: u8,
    pub size: u32,
    pub cu_count: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CacheType {
    Data,
    Instruction,
    Cpu,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RopInfo {
    pub unit_count: u32,
    pub operations_factor: u32,
    pub operations_count: u32,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct IntelDrmInfo {
    pub execution_units: Option<u32>,
    pub subslices: Option<u32>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DrmMemoryInfo {
    pub cpu_accessible_used: u64,
    pub cpu_accessible_total: u64,
    pub resizeable_bar: Option<bool>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct ClocksInfo {
    pub max_sclk: Option<i32>,
    pub max_mclk: Option<i32>,
    pub max_voltage: Option<i32>,
    pub table: Option<ClocksTable>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum ClocksTable {
    Amd(AmdClocksTableGen),
    Nvidia(NvidiaClocksTable),
    Intel(IntelClocksTable),
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct NvidiaClocksTable {
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub gpu_offsets: IndexMap<u32, NvidiaClockOffset>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub mem_offsets: IndexMap<u32, NvidiaClockOffset>,
    #[serde(default)]
    pub gpu_locked_clocks: Option<(u32, u32)>,
    #[serde(default)]
    pub vram_locked_clocks: Option<(u32, u32)>,
    #[serde(default)]
    pub gpu_clock_range: Option<(u32, u32)>,
    #[serde(default)]
    pub vram_clock_range: Option<(u32, u32)>,
}

/// Doc from `xe_gt_freq.c`
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq, Eq)]
pub struct IntelClocksTable {
    pub gt_freq: Option<(u64, u64)>,
    /// - rpn_freq: The Render Performance (RP) N level, which is the minimal one.
    pub rpn_freq: Option<u64>,
    /// - rpe_freq: The Render Performance (RP) E level, which is the efficient one.
    pub rpe_freq: Option<u64>,
    /// - rp0_freq: The Render Performance (RP) 0 level, which is the maximum one.
    pub rp0_freq: Option<u64>,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct NvidiaClockOffset {
    pub current: i32,
    pub min: i32,
    pub max: i32,
}

impl From<AmdClocksTableGen> for ClocksInfo {
    fn from(table: AmdClocksTableGen) -> Self {
        let max_sclk = table.get_max_sclk();
        let max_mclk = table.get_max_mclk();
        let max_voltage = table.get_max_sclk_voltage();
        Self {
            max_sclk,
            max_mclk,
            max_voltage,
            table: Some(ClocksTable::Amd(table)),
        }
    }
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct LinkInfo {
    pub current_width: Option<String>,
    pub current_speed: Option<String>,
    pub max_width: Option<String>,
    pub max_speed: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct VulkanInfo {
    pub device_name: String,
    pub api_version: String,
    pub driver: VulkanDriverInfo,
    pub enabled_layers: Vec<String>,
    pub features: IndexMap<String, bool>,
    pub extensions: IndexMap<String, bool>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct VulkanDriverInfo {
    pub version: u32,
    pub name: Option<String>,
    pub info: Option<String>,
    pub driver_version: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct OpenCLInfo {
    pub platform_name: String,
    pub device_name: String,
    pub version: String,
    pub driver_version: String,
    pub c_version: String,
    pub compute_units: u32,
    pub workgroup_size: usize,
    pub global_memory: u64,
    pub local_memory: u64,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PciInfo {
    pub vendor_id: String,
    pub vendor: Option<String>,
    pub model_id: String,
    pub model: Option<String>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DeviceStats {
    pub fan: FanStats,
    pub clockspeed: ClockspeedStats,
    pub voltage: VoltageStats,
    pub vram: VramStats,
    pub power: PowerStats,
    pub temps: HashMap<String, TemperatureEntry>,
    pub busy_percent: Option<u8>,
    pub performance_level: Option<PerformanceLevel>,
    pub core_power_state: Option<usize>,
    pub memory_power_state: Option<usize>,
    pub pcie_power_state: Option<usize>,
    pub throttle_info: Option<BTreeMap<String, Vec<String>>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TemperatureEntry {
    #[serde(flatten)]
    pub value: Temperature,
    /// If the temperature can be used for fan control
    #[serde(default)]
    pub display_only: bool,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct FanStats {
    pub control_enabled: bool,
    pub control_mode: Option<FanControlMode>,
    pub static_speed: Option<f32>,
    pub curve: Option<FanCurveMap>,
    pub pwm_current: Option<u8>,
    pub speed_current: Option<u32>,
    pub speed_max: Option<u32>,
    pub speed_min: Option<u32>,
    pub pwm_max: Option<u32>,
    pub pwm_min: Option<u32>,
    pub temperature_range: Option<(i32, i32)>,
    pub temperature_key: Option<String>,
    pub spindown_delay_ms: Option<u64>,
    pub change_threshold: Option<u64>,
    /// Nvidia-only
    pub auto_threshold: Option<u64>,
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
    pub zero_rpm_enable: Option<bool>,
    pub zero_rpm_temperature: Option<FanInfo>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ClockspeedStats {
    pub gpu_clockspeed: Option<u64>,
    /// Target clock
    #[serde(alias = "current_gfxclk")]
    pub target_gpu_clockspeed: Option<u64>,
    pub vram_clockspeed: Option<u64>,
    #[serde(default)]
    pub sensors: HashMap<String, u64>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct VoltageStats {
    pub gpu: Option<u64>,
    #[serde(default)]
    pub sensors: HashMap<String, u64>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default)]
pub struct VramStats {
    pub total: Option<u64>,
    pub used: Option<u64>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PowerStats {
    pub average: Option<f64>,
    pub current: Option<f64>,
    pub cap_current: Option<f64>,
    pub cap_max: Option<f64>,
    pub cap_min: Option<f64>,
    pub cap_default: Option<f64>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub sensors: HashMap<String, f64>,
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

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct PowerState {
    pub enabled: bool,
    pub min_value: Option<u64>,
    pub value: u64,
    pub index: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmdgpuParamsConfigurator {
    /// Enables overdrive by creating a modprobe.d file and regenerating the initramfs
    Modprobe(Option<InitramfsType>),
    BootArg(BootArgConfigurator),
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitramfsType {
    Debian,
    Mkinitcpio,
    Dracut,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootArgConfigurator {
    RpmOstree,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PmfwOptions {
    pub acoustic_limit: Option<u32>,
    pub acoustic_target: Option<u32>,
    pub minimum_pwm: Option<u32>,
    pub target_temperature: Option<u32>,
    pub zero_rpm: Option<bool>,
    pub zero_rpm_threshold: Option<u32>,
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
    pub static_speed: Option<f32>,
    pub curve: Option<FanCurveMap>,
    #[serde(default)]
    pub pmfw: PmfwOptions,
    pub spindown_delay_ms: Option<u64>,
    pub change_threshold: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ProfilesInfo {
    pub profiles: IndexMap<String, Option<ProfileRule>>,
    #[serde(default)]
    pub profile_hooks: IndexMap<String, ProfileHooks>,
    pub current_profile: Option<String>,
    pub auto_switch: bool,
    pub watcher_state: Option<ProfileWatcherState>,
}

impl PartialEq for ProfilesInfo {
    fn eq(&self, other: &Self) -> bool {
        self.profiles.as_slice() == other.profiles.as_slice()
            && self.profile_hooks.as_slice() == other.profile_hooks.as_slice()
            && self.current_profile == other.current_profile
            && self.auto_switch == other.auto_switch
    }
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type", content = "filter", rename_all = "lowercase")]
pub enum ProfileRule {
    Process(ProcessProfileRule),
    Gamemode(Option<ProcessProfileRule>),
    And(Vec<ProfileRule>),
    Or(Vec<ProfileRule>),
}

impl Default for ProfileRule {
    fn default() -> Self {
        Self::Process(ProcessProfileRule::default())
    }
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ProcessProfileRule {
    pub name: Arc<str>,
    pub args: Option<String>,
}

impl Default for ProcessProfileRule {
    fn default() -> Self {
        Self {
            name: String::new().into(),
            args: None,
        }
    }
}

pub type ProfileProcessMap = IndexMap<i32, ProfileProcessInfo>;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct ProfileWatcherState {
    pub process_list: ProfileProcessMap,
    pub gamemode_games: IndexSet<i32>,
    pub process_names_map: HashMap<Arc<str>, HashSet<i32>>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProfileProcessInfo {
    pub name: Arc<str>,
    pub cmdline: Box<str>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ProcessList {
    pub processes: BTreeMap<u32, ProcessInfo>,
    pub supported_util_types: HashSet<ProcessUtilizationType>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcessInfo {
    pub name: String,
    pub args: String,
    pub memory_used: u64,
    pub types: Vec<ProcessType>,
    pub util: HashMap<ProcessUtilizationType, u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProcessUtilizationType {
    Graphics,
    Compute,
    Memory,
    Encode,
    Decode,
}

impl ProcessUtilizationType {
    pub const ALL: &[ProcessUtilizationType] = &[
        ProcessUtilizationType::Graphics,
        ProcessUtilizationType::Compute,
        ProcessUtilizationType::Memory,
        ProcessUtilizationType::Encode,
        ProcessUtilizationType::Decode,
    ];
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum ProcessType {
    Graphics,
    Compute,
}
