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
    ClockspeedStats, DeviceInfo, DeviceStats, FanStats, GpuPciInfo, LinkInfo, PciInfo, PowerStats,
    VoltageStats, VramStats,
};
use pciid_parser::Database;
use std::{
    borrow::Cow,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{select, sync::Notify, task::JoinHandle, time::sleep};
use tracing::{debug, error, info, warn};

type FanControlHandle = (Arc<Notify>, JoinHandle<()>);

pub struct GpuController {
    pub handle: GpuHandle,
    pub pci_info: Option<GpuPciInfo>,
    pub fan_control_handle: Arc<Mutex<Option<FanControlHandle>>>,
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
            fan_control_handle: Arc::new(Mutex::new(None)),
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

        DeviceInfo {
            pci_info,
            vulkan_info,
            driver,
            vbios_version,
            link_info,
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
        Ok(DeviceStats {
            fan_stats: FanStats {
                fan_speed_current: self.hw_mon_and_then(HwMon::get_fan_current),
                fan_speed_max: self.hw_mon_and_then(HwMon::get_fan_max),
                fan_speed_min: self.hw_mon_and_then(HwMon::get_fan_min),
                fan_control_enabled: self
                    .fan_control_handle
                    .lock()
                    .map_err(|err| anyhow!("Could not lock fan control mutex: {err}"))?
                    .is_some(),
            },
            clockspeed_stats: ClockspeedStats {
                gpu_clockspeed: self.hw_mon_and_then(HwMon::get_gpu_clockspeed),
                vram_clockspeed: self.hw_mon_and_then(HwMon::get_vram_clockspeed),
            },
            voltage_stats: VoltageStats {
                gpu_voltage: self.hw_mon_and_then(HwMon::get_gpu_voltage),
                northbridge_voltage: self.hw_mon_and_then(HwMon::get_northbridge_voltage),
            },
            vram_stats: VramStats {
                total_vram: self.handle.get_total_vram().ok(),
                used_vram: self.handle.get_used_vram().ok(),
            },
            power_stats: PowerStats {
                power_average: self.hw_mon_and_then(HwMon::get_power_average),
                power_cap_current: self.hw_mon_and_then(HwMon::get_power_cap),
                power_cap_max: self.hw_mon_and_then(HwMon::get_power_cap_max),
                power_cap_min: self.hw_mon_and_then(HwMon::get_power_cap_min),
                power_cap_default: self.hw_mon_and_then(HwMon::get_power_cap_default),
            },
            temps: self.hw_mon_map(HwMon::get_temps).unwrap_or_default(),
            busy_percent: self.handle.get_busy_percent().ok(),
            performance_level: self.handle.get_power_force_performance_level().ok(),
        })
    }

    fn hw_mon_and_then<U>(&self, f: fn(&HwMon) -> Result<U, Error>) -> Option<U> {
        self.handle.hw_monitors.first().and_then(|mon| f(mon).ok())
    }

    fn hw_mon_map<U>(&self, f: fn(&HwMon) -> U) -> Option<U> {
        self.handle.hw_monitors.first().map(f)
    }

    pub fn start_fan_control(
        &self,
        curve: FanCurve,
        temp_key: String,
        interval: Duration,
    ) -> anyhow::Result<()> {
        let hw_mon = self
            .handle
            .hw_monitors
            .first()
            .cloned()
            .context("This GPU has no monitor")?;
        hw_mon
            .set_fan_control_method(FanControlMethod::Manual)
            .context("Could not set fan control method")?;

        let max_rpm = hw_mon.get_fan_max().context("Could not get min RPM")?;
        let min_rpm = hw_mon.get_fan_min().context("Could not get max RPM")?;

        let mut notify_guard = self
            .fan_control_handle
            .lock()
            .map_err(|err| anyhow!("Lock error: {err}"))?;

        if notify_guard.is_some() {
            return Ok(());
        }

        let notify = Arc::new(Notify::new());
        let task_notify = notify.clone();

        let notify_handle = self.fan_control_handle.clone();
        let handle = tokio::spawn(async move {
            loop {
                let mut temps = hw_mon.get_temps();
                let temp = temps
                    .remove(&temp_key)
                    .expect("Could not get temperature by given key");
                let target_rpm = curve.rpm_at_temp(temp, min_rpm, max_rpm);
                debug!("Fan control tick: setting rpm to {target_rpm}");

                if let Err(err) = hw_mon.set_fan_target(target_rpm) {
                    error!("Could not set fan speed: {err}, disabling fan control");
                    break;
                }

                select! {
                    _ = sleep(interval) => (),
                    _ = task_notify.notified() => break,
                }
            }
            info!("Shutting down fan control");
            if let Err(err) = hw_mon.set_fan_control_method(FanControlMethod::Auto) {
                error!("Could not set fan control back to automatic: {err}");
            }
            notify_handle
                .lock()
                .expect("Fan control mutex error")
                .take();
        });

        *notify_guard = Some((notify, handle));

        info!(
            "Started fan control with interval {}ms",
            interval.as_millis()
        );

        Ok(())
    }

    pub async fn stop_fan_control(&self) -> anyhow::Result<()> {
        let maybe_notify = self
            .fan_control_handle
            .lock()
            .map_err(|err| anyhow!("Lock error: {err}"))?
            .take();
        if let Some((notify, handle)) = maybe_notify {
            notify.notify_one();
            handle.await?;
        }
        Ok(())
    }
}
