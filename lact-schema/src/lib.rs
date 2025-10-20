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
pub use response::ResponseData;
#[cfg(feature = "schema")]
use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};

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
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct Pong;

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct SystemInfo {
    pub version: String,
    pub commit: Option<String>,
    pub profile: String,
    pub distro: Option<String>,
    pub kernel_version: String,
    pub amdgpu_overdrive_enabled: Option<bool>,
    #[cfg_attr(
        feature = "schema",
        schemars(schema_with = "amdgpu_params_configurator_schema")
    )]
    pub amdgpu_params_configurator: Option<AmdgpuParamsConfigurator>,
}

#[cfg(feature = "schema")]
fn amdgpu_params_configurator_schema(_gen: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "oneOf": [
            {
                "type": "object",
                "properties": {
                    "Modprobe": {
                        "oneOf": [
                            {
                                "type": "null"
                            },
                            {
                                "type": "string",
                                "enum": ["debian", "mkinitcpio", "dracut"]
                            }
                        ]
                    },
                },
                "required": ["Modprobe"]
            },
            {
                "type": "object",
                "properties": {
                    "BootArg": {
                        "oneOf": [
                            {
                                "type": "null"
                            },
                            {
                                "type": "string",
                                "enum": ["rpm-ostree"]
                            }
                        ]
                    },
                },
                "required": ["BootArg"]
            }
        ]
    })
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct GpuPciInfo {
    pub device_pci_info: PciInfo,
    pub subsystem_pci_info: PciInfo,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone, Copy)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub enum DeviceFlag {
    ConfigurableFanControl,
    DumpableVBios,
    HasPmfw,
    AutoFanThreshold,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct DeviceInfo {
    pub pci_info: Option<GpuPciInfo>,
    #[serde(default)]
    pub vulkan_instances: Vec<VulkanInfo>,
    pub opencl_info: Option<OpenCLInfo>,
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
                    .map(|size| format!("{} MiB", size / 1024 / 1024)),
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
                    Some((memory_info.cpu_accessible_total / 1024 / 1024).to_string()),
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
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub enum CacheInfo {
    Amd(Vec<(AmdCacheInstance, u16)>),
    Nvidia { l2: u32 },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct AmdCacheInstance {
    pub types: Vec<CacheType>,
    pub level: u8,
    pub size: u32,
    pub cu_count: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub enum CacheType {
    Data,
    Instruction,
    Cpu,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct RopInfo {
    pub unit_count: u32,
    pub operations_factor: u32,
    pub operations_count: u32,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct IntelDrmInfo {
    pub execution_units: Option<u32>,
    pub subslices: Option<u32>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct DrmMemoryInfo {
    pub cpu_accessible_used: u64,
    pub cpu_accessible_total: u64,
    pub resizeable_bar: Option<bool>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct ClocksInfo {
    pub max_sclk: Option<i32>,
    pub max_mclk: Option<i32>,
    pub max_voltage: Option<i32>,
    pub table: Option<ClocksTable>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum ClocksTable {
    #[cfg_attr(feature = "schema", schemars(schema_with = "amd_clocks_table_schema"))]
    Amd(AmdClocksTableGen),
    Nvidia(NvidiaClocksTable),
    Intel(IntelClocksTable),
}

#[cfg(feature = "schema")]
fn amd_clocks_table_schema(gen: &mut SchemaGenerator) -> Schema {
    let range_schema = json_schema!({
        "type": "object",
        "properties": {
            "min": gen.subschema_for::<Option<i32>>(),
            "max": gen.subschema_for::<Option<i32>>(),
        }
    });

    let optional_range_schema = json_schema!({
        "oneOf": [
            {
                "type": "null"
            },
            range_schema
        ]
    });

    let clocks_level_schema = json_schema!({
        "type": "object",
        "properties": {
            "clockspeed": gen.subschema_for::<i32>(),
            "voltage": gen.subschema_for::<i32>(),
        }
    });

    let gcn_schema = json_schema!({
        "type": "object",
        "properties": {
            "sclk_levels": {
                "type": "array",
                "items": clocks_level_schema
            },
            "mclk_levels": {
                "type": "array",
                "items": clocks_level_schema
            },
            "od_range": {
                "type": "object",
                "properties": {
                    "sclk": range_schema,
                    "mclk": optional_range_schema,
                    "vddc": optional_range_schema,
                    "curve_sclk_points": {
                        "type": "array",
                        "items": range_schema,
                    },
                    "curve_voltage_points": {
                        "type": "array",
                        "items": range_schema,
                    },
                    "voltage_offset": optional_range_schema,
                }
            }
        }
    }
    );

    let rdna_schema = json_schema!({
    "type": "object",
    "properties": {
        "current_sclk_range": range_schema,
        "current_mclk_range": range_schema,
        "sclk_offset": optional_range_schema,
        "voltage_offset": optional_range_schema,
        "vddc_curve": {
            "type": "array",
            "items": clocks_level_schema
        },
        "od_range": {
            "type": "object",
            "properties": {
                "sclk": range_schema,
                "mclk": optional_range_schema,
                "curve_sclk_points": {
                    "type": "array",
                    "items": range_schema,
                },
                "curve_voltage_points": {
                    "type": "array",
                    "items": range_schema,
                },
            }
        }
    }
    });

    json_schema!({
        "type": "object",
        "properties": {
            "kind": {
                "type": "string",
                "enum": ["gcn", "rdna"]
            },
            "value": {
                "oneOf": [
                    gcn_schema,
                    rdna_schema,
                ]
            }
        }
    })
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct LinkInfo {
    pub current_width: Option<String>,
    pub current_speed: Option<String>,
    pub max_width: Option<String>,
    pub max_speed: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct VulkanDriverInfo {
    pub version: u32,
    pub name: Option<String>,
    pub info: Option<String>,
    pub driver_version: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct PciInfo {
    pub vendor_id: String,
    pub vendor: Option<String>,
    pub model_id: String,
    pub model: Option<String>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct DeviceStats {
    pub fan: FanStats,
    pub clockspeed: ClockspeedStats,
    pub voltage: VoltageStats,
    pub vram: VramStats,
    pub power: PowerStats,
    #[cfg_attr(
        feature = "schema",
        schemars(schema_with = "temperatures_schema", default)
    )]
    pub temps: HashMap<String, Temperature>,
    pub busy_percent: Option<u8>,
    #[cfg_attr(
        feature = "schema",
        schemars(schema_with = "performance_level_schema", default)
    )]
    pub performance_level: Option<PerformanceLevel>,
    pub core_power_state: Option<usize>,
    pub memory_power_state: Option<usize>,
    pub pcie_power_state: Option<usize>,
    pub throttle_info: Option<BTreeMap<String, Vec<String>>>,
}

#[cfg(feature = "schema")]
fn temperatures_schema(_gen: &mut SchemaGenerator) -> Schema {
    let optional_float = json_schema!({
        "type": [
            "number",
            "null"
        ],
        "format": "float",
        "optional": true,
    });

    json_schema!({
        "type": "object",
        "additionalProperties": {
            "type": "object",
            "properties": {
                "current": optional_float,
                "crit": optional_float,
                "crit_hyst": optional_float,
            }
        }
    })
}

#[cfg(feature = "schema")]
fn performance_level_schema(_gen: &mut SchemaGenerator) -> Schema {
    use schemars::json_schema;

    let enum_values = [
        PerformanceLevel::Auto,
        PerformanceLevel::High,
        PerformanceLevel::Low,
        PerformanceLevel::Manual,
    ]
    .map(|value| serde_json::to_value(value).unwrap());

    json_schema!({
        "type": ["string" ,"null"],
        "enum": enum_values,
        "optional": true
    })
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct PmfwInfo {
    #[cfg_attr(feature = "schema", schemars(schema_with = "fan_info_schema", default))]
    pub acoustic_limit: Option<FanInfo>,
    #[cfg_attr(feature = "schema", schemars(schema_with = "fan_info_schema", default))]
    pub acoustic_target: Option<FanInfo>,
    #[cfg_attr(feature = "schema", schemars(schema_with = "fan_info_schema", default))]
    pub target_temp: Option<FanInfo>,
    #[cfg_attr(feature = "schema", schemars(schema_with = "fan_info_schema", default))]
    pub minimum_pwm: Option<FanInfo>,
    pub zero_rpm_enable: Option<bool>,
    #[cfg_attr(feature = "schema", schemars(schema_with = "fan_info_schema", default))]
    pub zero_rpm_temperature: Option<FanInfo>,
}

#[cfg(feature = "schema")]
fn fan_info_schema(gen: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "object",
        "properties": {
            "current": gen.subschema_for::<u32>(),
            "allowed_range": gen.subschema_for::<Option<(u32, u32)>>(),
        }
    })
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct ClockspeedStats {
    pub gpu_clockspeed: Option<u64>,
    /// Target clock
    pub current_gfxclk: Option<u64>,
    pub vram_clockspeed: Option<u64>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct VoltageStats {
    pub gpu: Option<u64>,
    pub northbridge: Option<u64>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct VramStats {
    pub total: Option<u64>,
    pub used: Option<u64>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct PowerStats {
    pub average: Option<f64>,
    pub current: Option<f64>,
    pub cap_current: Option<f64>,
    pub cap_max: Option<f64>,
    pub cap_min: Option<f64>,
    pub cap_default: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct ProfileWatcherState {
    pub process_list: ProfileProcessMap,
    pub gamemode_games: IndexSet<i32>,
    pub process_names_map: HashMap<Arc<str>, HashSet<i32>>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct ProfileProcessInfo {
    pub name: Arc<str>,
    pub cmdline: Box<str>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct ProcessList {
    pub processes: BTreeMap<u32, ProcessInfo>,
    pub supported_util_types: HashSet<ProcessUtilizationType>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct ProcessInfo {
    pub name: String,
    pub args: String,
    pub memory_used: u64,
    pub types: Vec<ProcessType>,
    pub util: HashMap<ProcessUtilizationType, u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub enum ProcessType {
    Graphics,
    Compute,
}
