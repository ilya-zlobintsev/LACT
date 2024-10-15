use crate::{config, server::vulkan::get_vulkan_info};

use super::GpuController;
use amdgpu_sysfs::{gpu_handle::power_profile_mode::PowerProfileModesTable, hw_mon::HwMon};
use anyhow::anyhow;
use futures::future::LocalBoxFuture;
use lact_schema::{
    ClocksInfo, DeviceInfo, DeviceStats, DrmInfo, DrmMemoryInfo, GpuPciInfo, LinkInfo, PowerStates,
};
use nvml_wrapper::{Device, Nvml};
use std::{
    borrow::Cow,
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

    fn get_stats(&self, _gpu_config: Option<&config::Gpu>) -> DeviceStats {
        DeviceStats::default()
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
