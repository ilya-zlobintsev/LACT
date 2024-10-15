use crate::{config, server::vulkan::get_vulkan_info};

use super::GpuController;
use amdgpu_sysfs::{
    gpu_handle::power_profile_mode::PowerProfileModesTable,
    hw_mon::{HwMon, Temperature},
};
use anyhow::anyhow;
use futures::future::LocalBoxFuture;
use lact_schema::{
    ClocksInfo, DeviceInfo, DeviceStats, DrmInfo, DrmMemoryInfo, FanStats, GpuPciInfo, LinkInfo,
    PmfwInfo, PowerStates, PowerStats, VramStats,
};
use nvml_wrapper::{
    enum_wrappers::device::{TemperatureSensor, TemperatureThreshold},
    Device, Nvml,
};
use std::{
    borrow::Cow,
    collections::HashMap,
    path::{Path, PathBuf},
    rc::Rc,
};
use tracing::{error, warn};

pub struct NvidiaGpuController {
    pub nvml: Rc<Nvml>,
    pub pci_slot_id: String,
    pub pci_info: GpuPciInfo,
    pub sysfs_path: PathBuf,
}

impl NvidiaGpuController {
    fn device(&self) -> Device<'_> {
        self.nvml
            .device_by_pci_bus_id(self.pci_slot_id.as_str())
            .expect("Can no longer get device")
    }
}

impl GpuController for NvidiaGpuController {
    fn get_id(&self) -> anyhow::Result<String> {
        let GpuPciInfo {
            device_pci_info,
            subsystem_pci_info,
        } = &self.pci_info;

        Ok(format!(
            "{}:{}-{}:{}-{}",
            device_pci_info.vendor_id,
            device_pci_info.model_id,
            subsystem_pci_info.vendor_id,
            subsystem_pci_info.model_id,
            self.pci_slot_id
        ))
    }

    fn get_pci_info(&self) -> Option<&GpuPciInfo> {
        Some(&self.pci_info)
    }

    fn get_path(&self) -> &Path {
        &self.sysfs_path
    }

    fn get_info(&self) -> DeviceInfo {
        let device = self.device();

        let vulkan_info = match get_vulkan_info(
            &self.pci_info.device_pci_info.vendor_id,
            &self.pci_info.device_pci_info.model_id,
        ) {
            Ok(info) => Some(info),
            Err(err) => {
                warn!("could not load vulkan info: {err}");
                None
            }
        };

        DeviceInfo {
            pci_info: Some(Cow::Borrowed(&self.pci_info)),
            vulkan_info,
            driver: format!(
                "nvidia {}",
                self.nvml.sys_driver_version().unwrap_or_default()
            ), // NVML should always be "nvidia"
            vbios_version: device
                .vbios_version()
                .map_err(|err| error!("could not get VBIOS version: {err}"))
                .ok(),
            link_info: LinkInfo::default(),
            drm_info: Some(DrmInfo {
                device_name: device.name().ok(),
                pci_revision_id: None,
                family_name: device.architecture().map(|arch| arch.to_string()).ok(),
                family_id: None,
                asic_name: None,
                chip_class: device.architecture().map(|arch| arch.to_string()).ok(),
                compute_units: None,
                vram_type: None,
                vram_clock_ratio: 1.0,
                vram_bit_width: device.current_pcie_link_width().ok(),
                vram_max_bw: None,
                l1_cache_per_cu: None,
                l2_cache: None,
                l3_cache_mb: None,
                memory_info: device
                    .bar1_memory_info()
                    .map(|info| DrmMemoryInfo {
                        cpu_accessible_used: info.used,
                        cpu_accessible_total: info.total,
                        resizeable_bar: None,
                    })
                    .ok(),
            }),
        }
    }

    fn hw_monitors(&self) -> &[HwMon] {
        &[]
    }

    fn get_pci_slot_name(&self) -> Option<String> {
        Some(self.pci_slot_id.clone())
    }

    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    fn get_stats(&self, gpu_config: Option<&config::Gpu>) -> DeviceStats {
        let device = self.device();

        let mut temps = HashMap::new();

        if let Ok(temp) = device.temperature(TemperatureSensor::Gpu) {
            let crit = device
                .temperature_threshold(TemperatureThreshold::Shutdown)
                .map(|value| value as f32)
                .ok();

            temps.insert(
                "GPU".to_owned(),
                Temperature {
                    current: Some(temp as f32),
                    crit,
                    crit_hyst: None,
                },
            );
        };

        let fan_settings = gpu_config.and_then(|config| config.fan_control_settings.as_ref());

        let pwm_current = if device.num_fans().is_ok_and(|num| num > 0) {
            device
                .fan_speed(0)
                .ok()
                .map(|value| (f64::from(value) * 2.55) as u8)
        } else {
            None
        };

        let vram = device
            .memory_info()
            .map(|info| VramStats {
                total: Some(info.total),
                used: Some(info.used),
            })
            .unwrap_or_default();

        DeviceStats {
            temps,
            fan: FanStats {
                control_enabled: gpu_config.is_some_and(|config| config.fan_control_enabled),
                control_mode: fan_settings.map(|settings| settings.mode),
                static_speed: fan_settings.map(|settings| settings.static_speed),
                curve: fan_settings.map(|settings| settings.curve.0.clone()),
                spindown_delay_ms: fan_settings.and_then(|settings| settings.spindown_delay_ms),
                change_threshold: fan_settings.and_then(|settings| settings.change_threshold),
                speed_current: None,
                speed_max: None,
                speed_min: None,
                pwm_current,
                pmfw_info: PmfwInfo::default(),
            },
            power: PowerStats {
                average: None,
                current: device
                    .power_usage()
                    .map(|mw| f64::from(mw) / 1_000_000.0)
                    .ok(),
                cap_current: device
                    .power_management_limit()
                    .map(|mw| f64::from(mw) / 1_000_000.0)
                    .ok(),
                cap_max: device
                    .power_management_limit_constraints()
                    .map(|constraints| f64::from(constraints.max_limit) / 1_000_000.0)
                    .ok(),
                cap_min: device
                    .power_management_limit_constraints()
                    .map(|constraints| f64::from(constraints.max_limit) / 1_000_000.0)
                    .ok(),
                cap_default: device
                    .power_management_limit_default()
                    .map(|mw| f64::from(mw) / 1_000_000.0)
                    .ok(),
            },
            busy_percent: device
                .utilization_rates()
                .map(|utilization| u8::try_from(utilization.gpu).expect("Invalid percentage"))
                .ok(),
            vram,
            ..Default::default()
        }
    }

    fn get_clocks_info(&self) -> anyhow::Result<ClocksInfo> {
        Ok(ClocksInfo::default())
    }

    fn get_power_states(&self, _gpu_config: Option<&config::Gpu>) -> PowerStates {
        PowerStates::default()
    }

    fn get_power_profile_modes(&self) -> anyhow::Result<PowerProfileModesTable> {
        Err(anyhow!("Not supported on Nvidia"))
    }

    fn reset_pmfw_settings(&self) {}

    fn vbios_dump(&self) -> anyhow::Result<Vec<u8>> {
        Err(anyhow!("Not supported on Nvidia"))
    }

    fn apply_config<'a>(
        &'a self,
        _config: &'a config::Gpu,
    ) -> LocalBoxFuture<'a, anyhow::Result<()>> {
        Box::pin(async { Ok(()) })
    }

    fn cleanup_clocks(&self) -> anyhow::Result<()> {
        Ok(())
    }
}
