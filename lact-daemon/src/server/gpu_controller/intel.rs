use super::GpuController;
use crate::{config, server::vulkan::get_vulkan_info};
use amdgpu_sysfs::gpu_handle::power_profile_mode::PowerProfileModesTable;
use anyhow::anyhow;
use futures::future::LocalBoxFuture;
use lact_schema::{
    ClocksInfo, ClockspeedStats, DeviceInfo, DeviceStats, GpuPciInfo, LinkInfo, PowerStates,
};
use std::{
    fmt::Display,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};
use tracing::{error, info, warn};

enum DriverType {
    Xe,
    I915,
}

pub struct IntelGpuController {
    sysfs_path: PathBuf,
    driver: String,
    driver_type: DriverType,
    pci_slot_id: Option<String>,
    pci_info: GpuPciInfo,
    tile_gts: Vec<PathBuf>,
}

impl IntelGpuController {
    pub fn new(
        sysfs_path: PathBuf,
        driver: String,
        pci_slot_id: Option<String>,
        pci_info: GpuPciInfo,
    ) -> Self {
        let driver_type = match driver.as_str() {
            "xe" => DriverType::Xe,
            "i915" => DriverType::I915,
            _ => unreachable!(),
        };

        let mut tile_gts = vec![];

        if let DriverType::Xe = driver_type {
            for entry in fs::read_dir(&sysfs_path).into_iter().flatten().flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.starts_with("tile") {
                        for gt_entry in fs::read_dir(entry.path()).into_iter().flatten().flatten() {
                            if let Some(gt_name) = gt_entry.file_name().to_str() {
                                if gt_name.starts_with("gt") {
                                    tile_gts.push(gt_entry.path());
                                }
                            }
                        }
                    }
                }
            }

            info!(
                "initialized {} gt at '{}'",
                tile_gts.len(),
                sysfs_path.display()
            );
        }

        Self {
            sysfs_path,
            driver,
            driver_type,
            pci_slot_id,
            pci_info,
            tile_gts,
        }
    }
}

impl GpuController for IntelGpuController {
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
            self.pci_slot_id.as_deref().unwrap_or("unknown-slot")
        ))
    }

    fn get_pci_info(&self) -> Option<&GpuPciInfo> {
        Some(&self.pci_info)
    }

    fn get_path(&self) -> &Path {
        &self.sysfs_path
    }

    fn get_info(&self, include_vulkan: bool) -> DeviceInfo {
        let vulkan_info = if include_vulkan {
            match get_vulkan_info(
                &self.pci_info.device_pci_info.vendor_id,
                &self.pci_info.device_pci_info.model_id,
            ) {
                Ok(info) => Some(info),
                Err(err) => {
                    warn!("could not load vulkan info: {err}");
                    None
                }
            }
        } else {
            None
        };

        DeviceInfo {
            pci_info: Some(self.pci_info.clone()),
            vulkan_info,
            driver: self.driver.clone(),
            vbios_version: None,
            link_info: LinkInfo::default(),
            drm_info: None,
        }
    }

    fn get_pci_slot_name(&self) -> Option<String> {
        self.pci_slot_id.clone()
    }

    fn apply_config<'a>(
        &'a self,
        _config: &'a config::Gpu,
    ) -> LocalBoxFuture<'a, anyhow::Result<()>> {
        Box::pin(async { Ok(()) })
    }

    fn get_stats(&self, _gpu_config: Option<&config::Gpu>) -> DeviceStats {
        let current_gfxclk = self.read_gt_file("freq0/cur_freq");

        let clockspeed = ClockspeedStats {
            gpu_clockspeed: self
                .read_gt_file("freq0/act_freq")
                .filter(|freq| *freq != 0)
                .or_else(|| current_gfxclk.map(u64::from)),
            current_gfxclk,
            vram_clockspeed: None,
        };

        DeviceStats {
            clockspeed,
            ..Default::default()
        }
    }

    fn get_clocks_info(&self) -> anyhow::Result<ClocksInfo> {
        Err(anyhow!("Not supported"))
    }

    fn get_power_states(&self, _gpu_config: Option<&config::Gpu>) -> PowerStates {
        PowerStates::default()
    }

    fn reset_pmfw_settings(&self) {}

    fn cleanup_clocks(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn get_power_profile_modes(&self) -> anyhow::Result<PowerProfileModesTable> {
        Err(anyhow!("Not supported"))
    }

    fn vbios_dump(&self) -> anyhow::Result<Vec<u8>> {
        Err(anyhow!("Not supported"))
    }
}

impl IntelGpuController {
    fn first_tile_gt(&self) -> Option<&Path> {
        self.tile_gts.first().map(PathBuf::as_ref)
    }

    fn read_gt_file<T>(&self, file_name: &str) -> Option<T>
    where
        T: FromStr,
        T::Err: Display,
    {
        if let Some(file_path) = self.first_tile_gt().map(|path| path.join(file_name)) {
            if file_path.exists() {
                match fs::read_to_string(&file_path) {
                    Ok(contents) => match contents.trim().parse() {
                        Ok(value) => return Some(value),
                        Err(err) => {
                            error!(
                                "could not parse value from '{}': {err}",
                                file_path.display()
                            );
                        }
                    },
                    Err(err) => {
                        error!("could not read file at '{}': {err}", file_path.display());
                    }
                }
            }
        }

        None
    }
}
