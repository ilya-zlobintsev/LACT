pub mod fan_control;

use self::fan_control::FanCurve;
use super::vulkan::get_vulkan_info;
use crate::fork::run_forked;
use amdgpu_sysfs::{
    error::Error,
    gpu_handle::GpuHandle,
    hw_mon::{FanControlMethod, HwMon},
};
use anyhow::{anyhow, Context};
use lact_schema::{
    ClocksInfo, ClockspeedStats, DeviceInfo, DeviceStats, FanStats, GpuPciInfo, LinkInfo, PciInfo,
    PowerStats, VoltageStats, VramStats,
};
use pciid_parser::Database;
use std::{
    borrow::Cow,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{select, sync::Notify, task::JoinHandle, time::sleep};
use tracing::{debug, error, info, trace, warn};

type FanControlHandle = (Arc<Notify>, JoinHandle<()>, FanCurve);

pub struct GpuController {
    pub handle: GpuHandle,
    pub pci_info: Option<GpuPciInfo>,
    pub fan_control_handle: Mutex<Option<FanControlHandle>>,
}

impl GpuController {
    pub fn new_from_path(sysfs_path: PathBuf) -> anyhow::Result<Self> {
        let handle = GpuHandle::new_from_path(sysfs_path)
            .map_err(|error| anyhow!("failed to initialize gpu handle: {error}"))?;

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
            pci_info,
            fan_control_handle: Mutex::new(None),
        })
    }

    pub fn get_info(&self) -> DeviceInfo {
        let vulkan_info = self.pci_info.as_ref().and_then(|pci_info| {
            match get_vulkan_info(
                &pci_info.device_pci_info.vendor_id,
                &pci_info.device_pci_info.model_id,
            ) {
                Ok(info) => Some(info),
                Err(err) => {
                    warn!("Could not load vulkan info: {err}");
                    None
                }
            }
        });
        let pci_info = self.pci_info.as_ref().map(Cow::Borrowed);
        let driver = self.handle.get_driver();
        let vbios_version = self.handle.get_vbios_version().ok();
        let link_info = self.get_link_info();
        let clocks_table = self.handle.get_clocks_table().ok();
        let clocks_info = clocks_table
            .as_ref()
            .map_or_else(Default::default, ClocksInfo::from);

        DeviceInfo {
            pci_info,
            vulkan_info,
            driver,
            vbios_version,
            link_info,
            clocks_table,
            clocks_info,
        }
    }

    fn get_link_info(&self) -> LinkInfo {
        LinkInfo {
            current_width: self.handle.get_current_link_width().ok(),
            current_speed: self.handle.get_current_link_speed().ok(),
            max_width: self.handle.get_max_link_width().ok(),
            max_speed: self.handle.get_max_link_speed().ok(),
        }
    }

    pub fn get_stats(&self) -> anyhow::Result<DeviceStats> {
        let fan_control_guard = self
            .fan_control_handle
            .lock()
            .map_err(|err| anyhow!("Could not lock fan control mutex: {err}"))?;

        Ok(DeviceStats {
            fan: FanStats {
                control_enabled: fan_control_guard.is_some(),
                curve: fan_control_guard
                    .as_ref()
                    .map(|(_, _, curve)| curve.0.clone()),
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

    fn hw_mon_and_then<U>(&self, f: fn(&HwMon) -> Result<U, Error>) -> Option<U> {
        self.handle.hw_monitors.first().and_then(|mon| f(mon).ok())
    }

    fn hw_mon_map<U>(&self, f: fn(&HwMon) -> U) -> Option<U> {
        self.handle.hw_monitors.first().map(f)
    }

    pub async fn start_fan_control(
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
        let task_curve = curve.clone();
        debug!("Using curve {curve:?}");

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
                let target_pwm = task_curve.pwm_at_temp(temp);
                trace!("Fan control tick: setting pwm to {target_pwm}");

                if let Err(err) = hw_mon.set_fan_pwm(target_pwm) {
                    error!("Could not set fan speed: {err}, disabling fan control");
                    break;
                }
            }
            info!("Exited fan control task");
        });

        *notify_guard = Some((notify, handle, curve));

        info!(
            "Started fan control with interval {}ms",
            interval.as_millis()
        );

        Ok(())
    }

    pub async fn stop_fan_control(&self, reset_mode: bool) -> anyhow::Result<()> {
        let maybe_notify = self
            .fan_control_handle
            .lock()
            .map_err(|err| anyhow!("Lock error: {err}"))?
            .take();
        if let Some((notify, handle, _)) = maybe_notify {
            notify.notify_one();
            handle.await?;
        }

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
        Ok(())
    }
}