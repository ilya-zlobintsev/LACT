pub mod fan_control;

use self::fan_control::FanCurve;
use super::vulkan::get_vulkan_info;
use crate::{config, fork::run_forked};
use anyhow::{anyhow, Context};
use lact_schema::{
    amdgpu_sysfs::{
        error::Error,
        gpu_handle::{
            overdrive::{ClocksTable, ClocksTableGen},
            GpuHandle, PerformanceLevel,
        },
        hw_mon::{FanControlMethod, HwMon},
        sysfs::SysFS,
    },
    ClocksInfo, ClockspeedStats, DeviceInfo, DeviceStats, DrmInfo, FanStats, GpuPciInfo, LinkInfo,
    PciInfo, PowerStats, VoltageStats, VramStats,
};
use pciid_parser::Database;
use std::{
    borrow::Cow,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{select, sync::Notify, task::JoinHandle, time::sleep};
use tracing::{debug, error, trace, warn};
#[cfg(feature = "libdrm_amdgpu_sys")]
use {
    lact_schema::DrmMemoryInfo,
    libdrm_amdgpu_sys::AMDGPU::{DeviceHandle as DrmHandle, GPU_INFO},
    std::{fs::File, os::fd::IntoRawFd},
};

type FanControlHandle = (Arc<Notify>, JoinHandle<()>);

pub struct GpuController {
    pub handle: GpuHandle,
    #[cfg(feature = "libdrm_amdgpu_sys")]
    pub drm_handle: Option<DrmHandle>,
    pub pci_info: Option<GpuPciInfo>,
    pub fan_control_handle: Mutex<Option<FanControlHandle>>,
}

impl GpuController {
    pub fn new_from_path(sysfs_path: PathBuf) -> anyhow::Result<Self> {
        let handle = GpuHandle::new_from_path(sysfs_path)
            .map_err(|error| anyhow!("failed to initialize gpu handle: {error}"))?;

        #[cfg(feature = "libdrm_amdgpu_sys")]
        let drm_handle = match get_drm_handle(&handle) {
            Ok(handle) => Some(handle),
            Err(err) => {
                warn!("Could not get DRM handle: {err}");
                None
            }
        };

        let mut device_pci_info = None;
        let mut subsystem_pci_info = None;

        if let Some((vendor_id, model_id)) = handle.get_pci_id() {
            device_pci_info = Some(PciInfo {
                vendor_id: vendor_id.to_owned(),
                vendor: None,
                model_id: model_id.to_owned(),
                model: None,
            });

            if let Some((subsys_vendor_id, subsys_model_id)) = handle.get_pci_subsys_id() {
                let (new_device_info, new_subsystem_info) = unsafe {
                    run_forked(|| {
                        let pci_db = Database::read().map_err(|err| err.to_string())?;
                        let pci_device_info = pci_db.get_device_info(
                            vendor_id,
                            model_id,
                            subsys_vendor_id,
                            subsys_model_id,
                        );

                        let device_pci_info = PciInfo {
                            vendor_id: vendor_id.to_owned(),
                            vendor: pci_device_info.vendor_name.map(str::to_owned),
                            model_id: model_id.to_owned(),
                            model: pci_device_info.device_name.map(str::to_owned),
                        };
                        let subsystem_pci_info = PciInfo {
                            vendor_id: subsys_vendor_id.to_owned(),
                            vendor: pci_device_info.subvendor_name.map(str::to_owned),
                            model_id: subsys_model_id.to_owned(),
                            model: pci_device_info.subdevice_name.map(str::to_owned),
                        };
                        Ok((device_pci_info, subsystem_pci_info))
                    })?
                };
                device_pci_info = Some(new_device_info);
                subsystem_pci_info = Some(new_subsystem_info);
            }
        }

        let pci_info = device_pci_info.and_then(|device_pci_info| {
            Some(GpuPciInfo {
                device_pci_info,
                subsystem_pci_info: subsystem_pci_info?,
            })
        });

        Ok(Self {
            handle,
            #[cfg(feature = "libdrm_amdgpu_sys")]
            drm_handle,
            pci_info,
            fan_control_handle: Mutex::new(None),
        })
    }

    pub fn get_id(&self) -> anyhow::Result<String> {
        let handle = &self.handle;
        let pci_id = handle.get_pci_id().context("Device has no vendor id")?;
        let pci_subsys_id = handle
            .get_pci_subsys_id()
            .context("Device has no subsys id")?;
        let pci_slot_name = handle
            .get_pci_slot_name()
            .context("Device has no pci slot")?;

        Ok(format!(
            "{}:{}-{}:{}-{}",
            pci_id.0, pci_id.1, pci_subsys_id.0, pci_subsys_id.1, pci_slot_name
        ))
    }

    pub fn get_path(&self) -> &Path {
        self.handle.get_path()
    }

    fn first_hw_mon(&self) -> anyhow::Result<&HwMon> {
        self.handle
            .hw_monitors
            .first()
            .context("GPU has no hardware monitor")
    }

    pub fn get_info(&self) -> DeviceInfo {
        let vulkan_info = self.pci_info.as_ref().and_then(|pci_info| {
            match get_vulkan_info(
                &pci_info.device_pci_info.vendor_id,
                &pci_info.device_pci_info.model_id,
            ) {
                Ok(info) => Some(info),
                Err(err) => {
                    warn!("could not load vulkan info: {err}");
                    None
                }
            }
        });
        let pci_info = self.pci_info.as_ref().map(Cow::Borrowed);
        let driver = self.handle.get_driver();
        let vbios_version = self.handle.get_vbios_version().ok();
        let link_info = self.get_link_info();
        let drm_info = self.get_drm_info();

        DeviceInfo {
            pci_info,
            vulkan_info,
            driver,
            vbios_version,
            link_info,
            drm_info,
        }
    }

    #[cfg(feature = "libdrm_amdgpu_sys")]
    fn get_drm_info(&self) -> Option<DrmInfo> {
        let drm_handle = self.drm_handle.as_ref();

        let drm_memory_info =
            drm_handle
                .and_then(|handle| handle.memory_info().ok())
                .map(|memory_info| DrmMemoryInfo {
                    cpu_accessible_used: memory_info.cpu_accessible_vram.heap_usage,
                    cpu_accessible_total: memory_info.cpu_accessible_vram.total_heap_size,
                });

        drm_handle
            .and_then(|handle| handle.device_info().ok())
            .map(|drm_info| DrmInfo {
                family_name: drm_info.get_family_name().to_string(),
                asic_name: drm_info.get_asic_name().to_string(),
                chip_class: drm_info.get_chip_class().to_string(),
                compute_units: drm_info.cu_active_number,
                vram_type: drm_info.get_vram_type().to_string(),
                vram_bit_width: drm_info.vram_bit_width,
                vram_max_bw: drm_info.peak_memory_bw_gb().to_string(),
                l2_cache: drm_info.calc_l2_cache_size(),
                memory_info: drm_memory_info,
            })
    }

    #[cfg(not(feature = "libdrm_amdgpu_sys"))]
    fn get_drm_info(&self) -> Option<DrmInfo> {
        None
    }

    fn get_link_info(&self) -> LinkInfo {
        LinkInfo {
            current_width: self.handle.get_current_link_width().ok(),
            current_speed: self.handle.get_current_link_speed().ok(),
            max_width: self.handle.get_max_link_width().ok(),
            max_speed: self.handle.get_max_link_speed().ok(),
        }
    }

    pub fn get_stats(&self, gpu_config: Option<&config::Gpu>) -> anyhow::Result<DeviceStats> {
        let fan_control_enabled = self
            .fan_control_handle
            .lock()
            .map_err(|err| anyhow!("Could not lock fan control mutex: {err}"))?
            .is_some();

        Ok(DeviceStats {
            fan: FanStats {
                control_enabled: fan_control_enabled,
                curve: gpu_config
                    .and_then(|config| config.fan_control_settings.as_ref())
                    .map(|settings| settings.curve.0.clone()),
                speed_current: self.hw_mon_and_then(HwMon::get_fan_current),
                speed_max: self.hw_mon_and_then(HwMon::get_fan_max),
                speed_min: self.hw_mon_and_then(HwMon::get_fan_min),
            },
            clockspeed: ClockspeedStats {
                gpu_clockspeed: self.hw_mon_and_then(HwMon::get_gpu_clockspeed),
                vram_clockspeed: self.hw_mon_and_then(HwMon::get_vram_clockspeed),
            },
            voltage: VoltageStats {
                gpu: self.hw_mon_and_then(HwMon::get_gpu_voltage),
                northbridge: self.hw_mon_and_then(HwMon::get_northbridge_voltage),
            },
            vram: VramStats {
                total: self.handle.get_total_vram().ok(),
                used: self.handle.get_used_vram().ok(),
            },
            power: PowerStats {
                average: self.hw_mon_and_then(HwMon::get_power_average),
                cap_current: self.hw_mon_and_then(HwMon::get_power_cap),
                cap_max: self.hw_mon_and_then(HwMon::get_power_cap_max),
                cap_min: self.hw_mon_and_then(HwMon::get_power_cap_min),
                cap_default: self.hw_mon_and_then(HwMon::get_power_cap_default),
            },
            temps: self.hw_mon_map(HwMon::get_temps).unwrap_or_default(),
            busy_percent: self.handle.get_busy_percent().ok(),
            performance_level: self.handle.get_power_force_performance_level().ok(),
            core_clock_levels: self.handle.get_core_clock_levels().ok(),
            memory_clock_levels: self.handle.get_memory_clock_levels().ok(),
            pcie_clock_levels: self.handle.get_pcie_clock_levels().ok(),
        })
    }

    pub fn get_clocks_info(&self) -> anyhow::Result<ClocksInfo> {
        let clocks_table = self
            .handle
            .get_clocks_table()
            .context("Clocks table not available")?;
        Ok(clocks_table.into())
    }

    fn hw_mon_and_then<U>(&self, f: fn(&HwMon) -> Result<U, Error>) -> Option<U> {
        self.handle.hw_monitors.first().and_then(|mon| f(mon).ok())
    }

    fn hw_mon_map<U>(&self, f: fn(&HwMon) -> U) -> Option<U> {
        self.handle.hw_monitors.first().map(f)
    }

    async fn start_fan_control(
        &self,
        curve: FanCurve,
        temp_key: String,
        interval: Duration,
    ) -> anyhow::Result<()> {
        // Stop existing task to re-apply new curve
        self.stop_fan_control(false).await?;

        let hw_mon = self
            .handle
            .hw_monitors
            .first()
            .cloned()
            .context("This GPU has no monitor")?;
        hw_mon
            .set_fan_control_method(FanControlMethod::Manual)
            .context("Could not set fan control method")?;

        let mut notify_guard = self
            .fan_control_handle
            .lock()
            .map_err(|err| anyhow!("Lock error: {err}"))?;

        let notify = Arc::new(Notify::new());
        let task_notify = notify.clone();

        let handle = tokio::spawn(async move {
            loop {
                select! {
                    _ = sleep(interval) => (),
                    _ = task_notify.notified() => break,
                }

                let mut temps = hw_mon.get_temps();
                let temp = temps
                    .remove(&temp_key)
                    .expect("Could not get temperature by given key");
                let target_pwm = curve.pwm_at_temp(temp);
                trace!("fan control tick: setting pwm to {target_pwm}");

                if let Err(err) = hw_mon.set_fan_pwm(target_pwm) {
                    error!("could not set fan speed: {err}, disabling fan control");
                    break;
                }
            }
            debug!("exited fan control task");
        });

        *notify_guard = Some((notify, handle));

        debug!(
            "started fan control with interval {}ms",
            interval.as_millis()
        );

        Ok(())
    }

    async fn stop_fan_control(&self, reset_mode: bool) -> anyhow::Result<()> {
        let maybe_notify = self
            .fan_control_handle
            .lock()
            .map_err(|err| anyhow!("Lock error: {err}"))?
            .take();
        if let Some((notify, handle)) = maybe_notify {
            notify.notify_one();
            handle.await?;

            if reset_mode {
                let hw_mon = self
                    .handle
                    .hw_monitors
                    .first()
                    .cloned()
                    .context("This GPU has no monitor")?;
                hw_mon
                    .set_fan_control_method(FanControlMethod::Auto)
                    .context("Could not set fan control back to automatic")?;
            }
        }

        Ok(())
    }

    pub async fn apply_config(&self, config: &config::Gpu) -> anyhow::Result<()> {
        if config.fan_control_enabled {
            if let Some(ref settings) = config.fan_control_settings {
                if settings.curve.0.is_empty() {
                    return Err(anyhow!("Cannot use empty fan curve"));
                }

                let interval = Duration::from_millis(settings.interval_ms);
                self.start_fan_control(
                    settings.curve.clone(),
                    settings.temperature_key.clone(),
                    interval,
                )
                .await?;
            } else {
                return Err(anyhow!(
                    "Trying to enable fan control with no settings provided"
                ));
            }
        } else {
            self.stop_fan_control(true).await?;
        }

        if let Some(cap) = config.power_cap {
            let hw_mon = self.first_hw_mon()?;
            hw_mon.set_power_cap(cap)?;
        } else if let Ok(hw_mon) = self.first_hw_mon() {
            if let Ok(default_cap) = hw_mon.get_power_cap_default() {
                hw_mon.set_power_cap(default_cap)?;
            }
        }

        if let Some(level) = config.performance_level {
            self.handle.set_power_force_performance_level(level)?;
        } else if self.handle.get_power_force_performance_level().is_ok() {
            self.handle
                .set_power_force_performance_level(PerformanceLevel::Auto)?;
        }

        if let Some(mode_index) = config.power_profile_mode_index {
            if config.performance_level != Some(PerformanceLevel::Manual) {
                return Err(anyhow!(
                    "Performance level has to be set to `manual` to use power profile modes"
                ));
            }

            self.handle.set_active_power_profile_mode(mode_index)?;
        }

        // Reset the clocks table in case the settings get reverted back to not having a clocks value configured
        self.handle.reset_clocks_table().ok();

        if config.is_core_clocks_used() {
            let mut table = self.handle.get_clocks_table()?;

            if let ClocksTableGen::Vega20(ref mut table) = table {
                // Avoid writing settings to the clocks table except the user-specified ones
                // There is an issue on some GPU models where the default values are actually outside of the allowed range
                // See https://github.com/sibradzic/amdgpu-clocks/issues/32#issuecomment-829953519 (part 2) for an example
                table.clear();

                table.voltage_offset = config.voltage_offset;
            }

            if let Some(min_clockspeed) = config.min_core_clock {
                table.set_min_sclk(min_clockspeed)?;
            }

            if let Some(min_clockspeed) = config.min_memory_clock {
                table.set_min_mclk(min_clockspeed)?;
            }

            if let Some(min_voltage) = config.min_voltage {
                table.set_min_voltage(min_voltage)?;
            }

            if let Some(clockspeed) = config.max_core_clock {
                table.set_max_sclk(clockspeed)?;
            }
            if let Some(clockspeed) = config.max_memory_clock {
                table.set_max_mclk(clockspeed)?;
            }
            if let Some(voltage) = config.max_voltage {
                table.set_max_voltage(voltage)?;
            }

            debug!("writing clocks commands: {:#?}", table.get_commands()?);

            self.handle
                .set_clocks_table(&table)
                .context("Could not write clocks table")?;
        }

        Ok(())
    }
}

#[cfg(feature = "libdrm_amdgpu_sys")]
fn get_drm_handle(handle: &GpuHandle) -> anyhow::Result<DrmHandle> {
    let slot_name = handle
        .get_pci_slot_name()
        .context("Device has no PCI slot name")?;
    let path = format!("/dev/dri/by-path/pci-{slot_name}-render");
    let drm_file =
        File::open(&path).with_context(|| format!("Could not open drm file at {path}"))?;
    let (handle, _, _) = DrmHandle::init(drm_file.into_raw_fd())
        .map_err(|err| anyhow!("Could not open drm handle, error code {err}"))?;
    Ok(handle)
}
