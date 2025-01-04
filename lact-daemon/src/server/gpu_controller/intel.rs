mod drm;

use super::GpuController;
use crate::{config, server::vulkan::get_vulkan_info};
use amdgpu_sysfs::gpu_handle::power_profile_mode::PowerProfileModesTable;
use anyhow::{anyhow, Context};
use drm::{bindings, i915};
use futures::future::LocalBoxFuture;
use lact_schema::{
    ClocksInfo, ClocksTable, ClockspeedStats, DeviceInfo, DeviceStats, DrmInfo, GpuPciInfo,
    IntelClocksTable, IntelDrmInfo, LinkInfo, PowerStates, VramStats,
};
use std::{
    cell::Cell,
    fmt::Display,
    fs,
    io::{BufRead, BufReader},
    os::{fd::AsRawFd, raw::c_int},
    path::{Path, PathBuf},
    str::FromStr,
    time::Instant,
};
use tracing::{debug, error, info, trace, warn};

enum DriverType {
    I915,
    Xe,
}

pub struct IntelGpuController {
    sysfs_path: PathBuf,
    driver: String,
    driver_type: DriverType,
    pci_slot_id: String,
    pci_info: GpuPciInfo,
    tile_gts: Vec<PathBuf>,
    drm_file: fs::File,
    last_gpu_busy: Cell<Option<(Instant, u64)>>,
}

impl IntelGpuController {
    pub fn new(
        sysfs_path: PathBuf,
        driver: String,
        pci_slot_id: String,
        pci_info: GpuPciInfo,
    ) -> anyhow::Result<Self> {
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
        let drm_file = if cfg!(not(test)) {
            let drm_path = format!("/dev/dri/by-path/pci-{pci_slot_id}-render");
            fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(drm_path)
                .context("Could not open DRM file")?
        } else {
            fs::File::open("/dev/null").unwrap()
        };

        Ok(Self {
            sysfs_path,
            driver,
            driver_type,
            pci_slot_id,
            pci_info,
            tile_gts,
            drm_file,
            last_gpu_busy: Cell::new(None),
        })
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
            self.pci_slot_id,
        ))
    }

    fn get_pci_info(&self) -> Option<&GpuPciInfo> {
        Some(&self.pci_info)
    }

    fn get_path(&self) -> &Path {
        &self.sysfs_path
    }

    fn get_info(&self) -> DeviceInfo {
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

        let drm_info = DrmInfo {
            intel: match self.driver_type {
                DriverType::I915 => self.get_drm_info_i915(),
                DriverType::Xe => self.get_drm_info_xe(),
            },
            vram_clock_ratio: 1.0,
            ..Default::default()
        };

        DeviceInfo {
            pci_info: Some(self.pci_info.clone()),
            vulkan_info,
            driver: self.driver.clone(),
            vbios_version: None,
            link_info: LinkInfo::default(),
            drm_info: Some(drm_info),
        }
    }

    fn get_pci_slot_name(&self) -> Option<String> {
        Some(self.pci_slot_id.clone())
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
        let current_gfxclk;
        let gpu_clockspeed;

        match self.driver_type {
            DriverType::Xe => {
                current_gfxclk = self.read_gt_file("freq0/cur_freq");
                gpu_clockspeed = self
                    .read_gt_file("freq0/act_freq")
                    .filter(|freq| *freq != 0)
                    .or_else(|| current_gfxclk.map(u64::from));
            }
            DriverType::I915 => {
                current_gfxclk = self.read_file("../gt_cur_freq_mhz");
                gpu_clockspeed = self.read_file("../gt_act_freq_mhz");
            }
        }

        let clockspeed = ClockspeedStats {
            gpu_clockspeed,
            current_gfxclk,
            vram_clockspeed: None,
        };

        DeviceStats {
            clockspeed,
            vram: VramStats {
                total: self
                    .drm_try_2(bindings::i915::drm_intel_get_aperture_sizes)
                    .map(|(_, total)| total as u64),
                used: None,
            },
            busy_percent: self.get_busy_percent(),
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
    fn debugfs_path(&self) -> PathBuf {
        #[cfg(test)]
        return PathBuf::from("/dev/null");

        Path::new("/sys/kernel/debug/dri").join(&self.pci_slot_id)
    }

    fn first_tile_gt(&self) -> Option<&Path> {
        self.tile_gts.first().map(PathBuf::as_ref)
    }

    /// Based on the input path, this has the following behaviour:
    /// - Basic relative paths are resolved relative to the sysfs device
    /// - Parent paths (starting with (../) are resolved on the sysfs card entry, without resolving device symlink
    fn sysfs_file_path(&self, path: impl AsRef<Path>) -> PathBuf {
        let path = path.as_ref();

        match path.strip_prefix("../") {
            Ok(path_relative_to_parent) => self
                .get_path()
                .parent()
                .expect("Device path has no parent")
                .join(path_relative_to_parent),
            Err(_) => self.get_path().join(path),
        }
    }

    fn read_file<T>(&self, path: impl AsRef<Path>) -> Option<T>
    where
        T: FromStr,
        T::Err: Display,
    {
        let file_path = self.sysfs_file_path(path);

        trace!("reading file from '{}'", file_path.display());

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
        let file_path = self.sysfs_file_path(path);

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

    fn get_drm_info_i915(&self) -> IntelDrmInfo {
        IntelDrmInfo {
            execution_units: self.drm_try(bindings::i915::drm_intel_get_eu_total),
            subslices: self.drm_try(bindings::i915::drm_intel_get_subslice_total),
        }
    }

    fn get_drm_info_xe(&self) -> IntelDrmInfo {
        IntelDrmInfo {
            execution_units: None,
            subslices: None,
        }
    }

    #[cfg_attr(test, allow(unreachable_code, unused_variables))]
    fn drm_try<T: Default>(&self, f: unsafe extern "C" fn(c_int, *mut T) -> c_int) -> Option<T> {
        #[cfg(test)]
        return None;

        unsafe {
            let mut out = T::default();
            let result = f(self.drm_file.as_raw_fd(), &mut out);
            if result == 0 {
                Some(out)
            } else {
                None
            }
        }
    }

    #[cfg_attr(test, allow(unreachable_code, unused_variables))]
    fn drm_try_2<T: Default, O: Default>(
        &self,
        f: unsafe extern "C" fn(c_int, *mut T, *mut O) -> c_int,
    ) -> Option<(T, O)> {
        #[cfg(test)]
        return None;

        unsafe {
            let mut a = T::default();
            let mut b = O::default();
            let result = f(self.drm_file.as_raw_fd(), &mut a, &mut b);
            if result == 0 {
                Some((a, b))
            } else {
                None
            }
        }
    }

    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation
    )]
    fn get_busy_percent(&self) -> Option<u8> {
        let path = self.debugfs_path().join("gt0/rps_boost");
        let file = fs::File::open(path).ok()?;
        let mut lines = BufReader::new(file).lines();

        while let Some(Ok(line)) = lines.next() {
            if let Some(contents) = line.strip_prefix("GPU busy?") {
                let raw_value = contents
                    .split_ascii_whitespace()
                    .last()?
                    .strip_suffix("ms")?;
                let gpu_busy: u64 = raw_value.parse().ok()?;
                let timestamp = Instant::now();

                if let Some((last_timestamp, last_gpu_busy)) =
                    self.last_gpu_busy.replace(Some((timestamp, gpu_busy)))
                {
                    let time_delta = timestamp - last_timestamp;
                    let gpu_busy_delta = gpu_busy - last_gpu_busy;

                    println!("time delta: {time_delta:?}, busy delta: {gpu_busy_delta}");

                    let percentage =
                        (gpu_busy_delta as f64 / time_delta.as_millis() as f64) * 100.0;
                    return Some(percentage as u8);
                }
            }
        }

        None
    }
}
