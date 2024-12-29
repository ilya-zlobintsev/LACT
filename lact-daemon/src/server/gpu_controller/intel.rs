use super::GpuController;
use crate::{config, server::vulkan::get_vulkan_info};
use amdgpu_sysfs::gpu_handle::power_profile_mode::PowerProfileModesTable;
use anyhow::{anyhow, Context};
use futures::future::LocalBoxFuture;
use lact_schema::{
    ClocksInfo, ClocksTable, ClockspeedStats, DeviceInfo, DeviceStats, GpuPciInfo,
    IntelClocksTable, LinkInfo, PowerStates,
};
use std::{
    fmt::Display,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};
use tracing::{debug, error, info, warn};

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

        for entry in fs::read_dir(&sysfs_path).into_iter().flatten().flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.starts_with("tile") {
                    for gt_entry in fs::read_dir(entry.path()).into_iter().flatten().flatten() {
                        if let Some(gt_name) = gt_entry.file_name().to_str() {
                            if gt_name.starts_with("gt") {
                                let gt_path = gt_entry
                                    .path()
                                    .strip_prefix(&sysfs_path)
                                    .unwrap()
                                    .to_owned();
                                debug!("initialized GT at '{}'", gt_path.display());
                                tile_gts.push(gt_path);
                            }
                        }
                    }
                }
            }
        }

        if !tile_gts.is_empty() {
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
        config: &'a config::Gpu,
    ) -> LocalBoxFuture<'a, anyhow::Result<()>> {
        Box::pin(async {
            match self.driver_type {
                DriverType::Xe => {
                    if let Some(max_clock) = config.clocks_configuration.max_core_clock {
                        self.write_gt_file("freq0/max_freq", &max_clock.to_string())
                            .context("Could not set max clock")?;
                    }
                    if let Some(min_clock) = config.clocks_configuration.min_core_clock {
                        self.write_gt_file("freq0/min_freq", &min_clock.to_string())
                            .context("Could not set min clock")?;
                    }
                }
                DriverType::I915 => {
                    if let Some(max_clock) = config.clocks_configuration.max_core_clock {
                        self.write_file("../gt_max_freq_mhz", &max_clock.to_string())
                            .context("Could not set max clock")?;
                    }
                    if let Some(min_clock) = config.clocks_configuration.min_core_clock {
                        self.write_file("../gt_min_freq_mhz", &min_clock.to_string())
                            .context("Could not set min clock")?;
                    }
                }
            }

            Ok(())
        })
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
        let clocks_table = match self.driver_type {
            DriverType::Xe => IntelClocksTable {
                gt_freq: self
                    .read_gt_file("freq0/min_freq")
                    .zip(self.read_gt_file("freq0/max_freq")),
                rp0_freq: self.read_gt_file("freq0/rp0_freq"),
                rpe_freq: self.read_gt_file("freq0/rpe_freq"),
                rpn_freq: self.read_gt_file("freq0/rpn_freq"),
            },
            DriverType::I915 => IntelClocksTable {
                gt_freq: self
                    .read_file("../gt_min_freq_mhz")
                    .zip(self.read_file("../gt_max_freq_mhz")),
                rpn_freq: self.read_file("../gt_RPn_freq_mhz"),
                rpe_freq: self.read_file("../gt_RP1_freq_mhz"),
                rp0_freq: self.read_file("../gt_RP0_freq_mhz"),
            },
        };

        let table = if clocks_table == IntelClocksTable::default() {
            None
        } else {
            Some(ClocksTable::Intel(clocks_table))
        };

        Ok(ClocksInfo {
            table,
            ..Default::default()
        })
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

    fn read_file<T>(&self, path: impl AsRef<Path>) -> Option<T>
    where
        T: FromStr,
        T::Err: Display,
    {
        let file_path = self.get_path().join(path);

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
        None
    }

    fn write_file(&self, path: impl AsRef<Path>, contents: &str) -> anyhow::Result<()> {
        let file_path = self.get_path().join(path);

        if file_path.exists() {
            fs::write(&file_path, contents)
                .with_context(|| format!("Could not write to '{}'", file_path.display()))?;
            Ok(())
        } else {
            Err(anyhow!("File '{}' does not exist", file_path.display()))
        }
    }

    fn read_gt_file<T>(&self, file_name: &str) -> Option<T>
    where
        T: FromStr,
        T::Err: Display,
    {
        self.first_tile_gt().and_then(|gt_path| {
            let file_path = gt_path.join(file_name);
            self.read_file(file_path)
        })
    }

    fn write_gt_file(&self, file_name: &str, contents: &str) -> anyhow::Result<()> {
        if let Some(gt_path) = self.first_tile_gt() {
            let file_path = gt_path.join(file_name);
            self.write_file(file_path, contents)
        } else {
            Err(anyhow!("No GTs available"))
        }
    }
}
