use crate::{
    config::{self, FanControlSettings},
    server::vulkan::get_vulkan_info,
};

use super::{fan_control::FanCurve, CommonControllerInfo, FanControlHandle, GpuController};
use amdgpu_sysfs::{gpu_handle::power_profile_mode::PowerProfileModesTable, hw_mon::Temperature};
use anyhow::{anyhow, Context};
use futures::future::LocalBoxFuture;
use lact_schema::{
    ClocksInfo, ClocksTable, ClockspeedStats, DeviceInfo, DeviceStats, DrmInfo, DrmMemoryInfo,
    FanControlMode, FanStats, IntelDrmInfo, LinkInfo, NvidiaClockInfo, NvidiaClocksTable, PmfwInfo,
    PowerState, PowerStates, PowerStats, VoltageStats, VramStats,
};
use nvml_wrapper::{
    bitmasks::device::ThrottleReasons,
    enum_wrappers::device::{Brand, Clock, TemperatureSensor, TemperatureThreshold},
    enums::device::DeviceArchitecture,
    Device, Nvml,
};
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    fmt::Write,
    rc::Rc,
    time::{Duration, Instant},
};
use tokio::{select, sync::Notify, time::sleep};
use tracing::{debug, error, trace, warn};

pub struct NvidiaGpuController {
    nvml: Rc<Nvml>,
    common: CommonControllerInfo,
    fan_control_handle: RefCell<Option<FanControlHandle>>,

    last_applied_gpc_offset: Cell<Option<i32>>,
    last_applied_mem_offset: Cell<Option<i32>>,
}

impl NvidiaGpuController {
    pub fn new(common: CommonControllerInfo, nvml: Rc<Nvml>) -> anyhow::Result<Self> {
        nvml.device_by_pci_bus_id(common.pci_slot_name.as_str())
            .with_context(|| {
                format!(
                    "Could not get PCI device '{}' from NVML",
                    common.pci_slot_name
                )
            })?;
        Ok(Self {
            nvml,
            common,
            fan_control_handle: RefCell::new(None),
            last_applied_gpc_offset: Cell::new(None),
            last_applied_mem_offset: Cell::new(None),
        })
    }

    fn device(&self) -> Device<'_> {
        self.nvml
            .device_by_pci_bus_id(self.common.pci_slot_name.as_str())
            .expect("Can no longer get device")
    }

    async fn start_curve_fan_control_task(
        &self,
        curve: FanCurve,
        settings: FanControlSettings,
    ) -> anyhow::Result<()> {
        // Stop existing task to re-apply new curve
        self.stop_fan_control().await?;

        let device = self.device();
        device
            .temperature(TemperatureSensor::Gpu)
            .context("Could not read temperature")?;

        let fan_count = device.num_fans().context("Could not read fan count")?;
        if fan_count == 0 {
            return Err(anyhow!("Device has no fans"));
        }

        let mut notify_guard = self
            .fan_control_handle
            .try_borrow_mut()
            .map_err(|err| anyhow!("Lock error: {err}"))?;

        let notify = Rc::new(Notify::new());
        let task_notify = notify.clone();

        let nvml = self.nvml.clone();
        let pci_slot_id = self.common.pci_slot_name.clone();
        debug!("spawning new fan control task");

        let handle = tokio::task::spawn_local(async move {
            let mut device = nvml
                .device_by_pci_bus_id(pci_slot_id.as_str())
                .expect("Can no longer get device");

            let mut last_pwm = (None, Instant::now());
            let mut last_temp = 0;

            let interval = Duration::from_millis(settings.interval_ms);
            let spindown_delay = Duration::from_millis(settings.spindown_delay_ms.unwrap_or(0));
            #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
            let change_threshold = settings.change_threshold.unwrap_or(0) as i32;

            loop {
                select! {
                    () = sleep(interval) => (),
                    () = task_notify.notified() => break,
                }

                #[allow(clippy::cast_possible_wrap)]
                let current_temp = device
                    .temperature(TemperatureSensor::Gpu)
                    .expect("Could not read temperature") as i32;

                if (last_temp - current_temp).abs() < change_threshold {
                    trace!("temperature changed from {last_temp}°C to {current_temp}°C, which is less than the {change_threshold}°C threshold, skipping speed adjustment");
                    continue;
                }

                let target_pwm = curve.pwm_at_temp(Temperature {
                    #[allow(clippy::cast_precision_loss)]
                    current: Some(current_temp as f32),
                    crit: None,
                    crit_hyst: None,
                });
                let now = Instant::now();

                if let (Some(previous_pwm), previous_timestamp) = last_pwm {
                    let diff = now - previous_timestamp;
                    if target_pwm < previous_pwm && diff < spindown_delay {
                        trace!(
                            "delaying fan spindown ({}ms left)",
                            (spindown_delay - diff).as_millis()
                        );
                        continue;
                    }
                }

                last_pwm = (Some(target_pwm), now);
                last_temp = current_temp;

                trace!("fan control tick: setting pwm to {target_pwm}");

                for fan in 0..fan_count {
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    if let Err(err) =
                        device.set_fan_speed(fan, (f64::from(target_pwm) / 2.5) as u32)
                    {
                        error!("could not set fan speed: {err}, disabling fan control");
                        break;
                    }
                }
            }
            debug!("exited fan control task");
        });

        *notify_guard = Some((notify, handle));

        debug!(
            "started fan control with interval {}ms",
            settings.interval_ms
        );

        Ok(())
    }

    async fn stop_fan_control(&self) -> anyhow::Result<()> {
        let mut fail_on_error = false;

        let maybe_notify = self
            .fan_control_handle
            .try_borrow_mut()
            .map_err(|err| anyhow!("Lock error: {err}"))?
            .take();
        if let Some((notify, handle)) = maybe_notify {
            notify.notify_one();
            handle.await?;
            fail_on_error = true;
        }

        let mut device = self.device();
        let fan_count = device.num_fans().context("Could not get fan count")?;
        for i in 0..fan_count {
            if let Err(err) = device
                .set_default_fan_speed(i)
                .context("Could not reset fan control to default")
            {
                if fail_on_error {
                    return Err(err);
                }
                error!("{err:#?}");
            }
        }

        Ok(())
    }

    fn try_get_power_states(&self) -> anyhow::Result<PowerStates> {
        let device = self.device();

        let supported_states = device
            .supported_performance_states()
            .context("Could not get supported pstates")?;

        let mut power_states = PowerStates::default();

        for pstate in supported_states {
            let (gpu_min, gpu_max) = device
                .min_max_clock_of_pstate(Clock::Graphics, pstate)
                .context("Could not read GPU pstates")?;

            power_states.core.push(PowerState {
                enabled: true,
                min_value: Some(u64::from(gpu_min)),
                value: u64::from(gpu_max),
                index: Some(
                    pstate
                        .as_c()
                        .try_into()
                        .expect("Power state always fits in u8"),
                ),
            });

            let (mem_min, mem_max) = device
                .min_max_clock_of_pstate(Clock::Memory, pstate)
                .context("Could not read memory pstates")?;

            power_states.vram.push(PowerState {
                enabled: true,
                min_value: Some(u64::from(mem_min)),
                value: u64::from(mem_max),
                index: Some(
                    pstate
                        .as_c()
                        .try_into()
                        .expect("Power state always fits in u8"),
                ),
            });
        }

        Ok(power_states)
    }

    // See https://github.com/ilya-zlobintsev/LACT/issues/418
    fn vram_offset_ratio(&self) -> i32 {
        let device = self.device();
        if let (Ok(brand), Ok(architecture)) = (device.brand(), device.architecture()) {
            let ratio = match (brand, architecture) {
                (Brand::GeForce, DeviceArchitecture::Ada) => 2,
                // TODO: check others
                _ => 1,
            };
            return ratio;
        }
        1
    }
}

impl GpuController for NvidiaGpuController {
    fn controller_info(&self) -> &CommonControllerInfo {
        &self.common
    }

    fn get_info(&self) -> DeviceInfo {
        let vulkan_info = match get_vulkan_info(&self.common.pci_info) {
            Ok(info) => Some(info),
            Err(err) => {
                warn!("could not load vulkan info: {err}");
                None
            }
        };

        let device = self.device();

        DeviceInfo {
            pci_info: Some(self.common.pci_info.clone()),
            vulkan_info,
            driver: format!(
                "nvidia {}",
                self.nvml.sys_driver_version().unwrap_or_default()
            ), // NVML should always be "nvidia"
            vbios_version: device
                .vbios_version()
                .map_err(|err| error!("could not get VBIOS version: {err}"))
                .ok(),
            link_info: LinkInfo {
                current_width: device.current_pcie_link_width().map(|v| v.to_string()).ok(),
                current_speed: device
                    .pcie_link_speed()
                    .map(|v| {
                        let mut output = format!("{} GT/s", v / 1000);
                        if let Ok(gen) = device.current_pcie_link_gen() {
                            let _ = write!(output, " PCIe gen {gen}");
                        }
                        output
                    })
                    .ok(),
                max_width: device.max_pcie_link_width().map(|v| v.to_string()).ok(),
                max_speed: device
                    .max_pcie_link_speed()
                    .ok()
                    .and_then(|v| v.as_integer())
                    .map(|v| {
                        let mut output = format!("{} GT/s", v / 1000);
                        if let Ok(gen) = device.current_pcie_link_gen() {
                            let _ = write!(output, " PCIe gen {gen}");
                        }
                        output
                    }),
            },
            drm_info: Some(DrmInfo {
                device_name: device.name().ok(),
                pci_revision_id: None,
                family_name: device.architecture().map(|arch| arch.to_string()).ok(),
                family_id: None,
                asic_name: None,
                chip_class: device.architecture().map(|arch| arch.to_string()).ok(),
                compute_units: None,
                cuda_cores: device.num_cores().ok(),
                vram_type: None,
                vram_clock_ratio: 1.0,
                vram_bit_width: device.current_pcie_link_width().ok(),
                vram_max_bw: None,
                l1_cache_per_cu: None,
                l2_cache: None,
                l3_cache_mb: None,
                memory_info: device
                    .bar1_memory_info()
                    .map(|bar_info| DrmMemoryInfo {
                        cpu_accessible_used: bar_info.used,
                        cpu_accessible_total: bar_info.total,
                        resizeable_bar: device
                            .memory_info()
                            .ok()
                            .map(|memory_info| bar_info.total >= memory_info.total),
                    })
                    .ok(),
                intel: IntelDrmInfo::default(),
            }),
        }
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

        let active_pstate = device
            .performance_state()
            .map(|pstate| pstate.as_c() as usize)
            .ok();

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
                current: device.power_usage().map(|mw| f64::from(mw) / 1000.0).ok(),
                cap_current: device
                    .power_management_limit()
                    .map(|mw| f64::from(mw) / 1000.0)
                    .ok(),
                cap_max: device
                    .power_management_limit_constraints()
                    .map(|constraints| f64::from(constraints.max_limit) / 1000.0)
                    .ok(),
                cap_min: device
                    .power_management_limit_constraints()
                    .map(|constraints| f64::from(constraints.min_limit) / 1000.0)
                    .ok(),
                cap_default: device
                    .power_management_limit_default()
                    .map(|mw| f64::from(mw) / 1000.0)
                    .ok(),
            },
            busy_percent: device
                .utilization_rates()
                .map(|utilization| u8::try_from(utilization.gpu).expect("Invalid percentage"))
                .ok(),
            vram,
            clockspeed: ClockspeedStats {
                gpu_clockspeed: device.clock_info(Clock::Graphics).map(Into::into).ok(),
                vram_clockspeed: device.clock_info(Clock::Memory).map(Into::into).ok(),
                current_gfxclk: None,
            },
            throttle_info: device.current_throttle_reasons().ok().map(|reasons| {
                reasons
                    .iter()
                    .filter(|reason| *reason != ThrottleReasons::GPU_IDLE)
                    .map(|reason| {
                        let mut name = String::new();
                        bitflags::parser::to_writer(&reason, &mut name).unwrap();
                        (name, vec![])
                    })
                    .collect()
            }),
            voltage: VoltageStats::default(), // Voltage reporting is not supported
            performance_level: None,
            core_power_state: active_pstate,
            memory_power_state: active_pstate,
            pcie_power_state: None,
        }
    }

    #[allow(clippy::cast_possible_wrap)]
    fn get_clocks_info(&self) -> anyhow::Result<ClocksInfo> {
        let device = self.device();

        let mut gpc = None;
        let mut mem = None;

        // Negative offset values are not correctly reported by NVML, so we have to use the last known applied value
        // instead of the actual read when an unreasonable value appears.

        if let Ok(max) = device.max_clock_info(Clock::Graphics) {
            if let Ok(offset_range) = device.gpc_clk_min_max_vf_offset() {
                if let Some(offset) = self
                    .last_applied_gpc_offset
                    .get()
                    .or_else(|| device.gpc_clk_vf_offset().ok())
                {
                    gpc = Some(NvidiaClockInfo {
                        max: max as i32,
                        offset,
                        offset_ratio: 1,
                        offset_range,
                    });
                }
            }
        }

        if let Ok(max) = device.max_clock_info(Clock::Memory) {
            if let Ok(offset_range) = device.mem_clk_min_max_vf_offset() {
                if let Some(offset) = self
                    .last_applied_mem_offset
                    .get()
                    .or_else(|| device.mem_clk_vf_offset().ok())
                {
                    mem = Some(NvidiaClockInfo {
                        max: max as i32,
                        offset,
                        offset_ratio: self.vram_offset_ratio(),
                        offset_range,
                    });
                }
            }
        }

        let table = NvidiaClocksTable { gpc, mem };

        Ok(ClocksInfo {
            table: Some(ClocksTable::Nvidia(table)),
            ..Default::default()
        })
    }

    fn get_power_states(&self, _gpu_config: Option<&config::Gpu>) -> PowerStates {
        self.try_get_power_states().unwrap_or_else(|err| {
            warn!("could not get pstates info: {err:#}");
            PowerStates::default()
        })
    }

    fn get_power_profile_modes(&self) -> anyhow::Result<PowerProfileModesTable> {
        Err(anyhow!("Not supported on Nvidia"))
    }

    fn reset_pmfw_settings(&self) {}

    fn vbios_dump(&self) -> anyhow::Result<Vec<u8>> {
        Err(anyhow!("Not supported on Nvidia"))
    }

    #[allow(clippy::cast_possible_wrap)]
    fn apply_config<'a>(
        &'a self,
        config: &'a config::Gpu,
    ) -> LocalBoxFuture<'a, anyhow::Result<()>> {
        Box::pin(async {
            let mut device = self.device();

            if let Some(cap) = config.power_cap {
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let cap = (cap * 1000.0) as u32;

                let current_cap = device
                    .power_management_limit()
                    .context("Could not get current cap")?;

                if current_cap != cap {
                    debug!("setting power cap to {cap}");
                    device
                        .set_power_management_limit(cap)
                        .context("Could not set power cap")?;
                }
            } else {
                let current_cap = device.power_management_limit();
                let default_cap = device.power_management_limit_default();

                if let (Ok(current_cap), Ok(default_cap)) = (current_cap, default_cap) {
                    if current_cap != default_cap {
                        debug!("resetting power cap to {default_cap}");
                        device
                            .set_power_management_limit(default_cap)
                            .context("Could not reset power cap")?;
                    }
                }
            }

            self.cleanup_clocks()?;

            if let Some(max_gpu_clock) = config.clocks_configuration.max_core_clock {
                let default_max_clock = device
                    .max_clock_info(Clock::Graphics)
                    .context("Could not read max graphics clock")?;
                let offset = max_gpu_clock - default_max_clock as i32;
                debug!(
                    "Using graphics clock offset {offset} (default max clock: {default_max_clock})"
                );

                device
                    .set_gpc_clk_vf_offset(offset)
                    .context("Could not set graphics clock offset")?;

                self.last_applied_gpc_offset.set(Some(offset));
            }

            if let Some(max_mem_clock) = config.clocks_configuration.max_memory_clock {
                let default_max_clock = device
                    .max_clock_info(Clock::Memory)
                    .context("Could not read max memory clock")?;
                let offset = (max_mem_clock - default_max_clock as i32) * self.vram_offset_ratio();
                debug!("Using mem clock offset {offset} (default max clock: {default_max_clock})");

                device
                    .set_mem_clk_vf_offset(offset)
                    .context("Could not set memory clock offset")?;

                self.last_applied_mem_offset.set(Some(offset));
            }

            if config.fan_control_enabled {
                let settings = config
                    .fan_control_settings
                    .as_ref()
                    .context("Fan control enabled with no settings")?;
                match settings.mode {
                    FanControlMode::Static => {
                        self.stop_fan_control()
                            .await
                            .context("Could not reset fan control")?;

                        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
                        let speed = (settings.static_speed * 100.0) as u32;

                        let fan_count = device.num_fans().context("Could not get fan count")?;
                        for fan in 0..fan_count {
                            device
                                .set_fan_speed(fan, speed)
                                .context("Could not reset fan speed to default")?;
                        }
                    }
                    FanControlMode::Curve => {
                        self.start_curve_fan_control_task(settings.curve.clone(), settings.clone())
                            .await?;
                    }
                }
            } else {
                self.stop_fan_control()
                    .await
                    .context("Could not reset fan control")?;
            }

            Ok(())
        })
    }

    fn cleanup_clocks(&self) -> anyhow::Result<()> {
        let device = self.device();

        if let Ok(current_offset) = device.gpc_clk_vf_offset() {
            if current_offset != 0 {
                device
                    .set_gpc_clk_vf_offset(0)
                    .context("Could not reset graphics clock offset")?;

                self.last_applied_gpc_offset.set(None);
            }
        }

        if let Ok(current_offset) = device.mem_clk_vf_offset() {
            if current_offset != 0 {
                device
                    .set_mem_clk_vf_offset(0)
                    .context("Could not reset memory clock offset")?;

                self.last_applied_mem_offset.set(None);
            }
        }

        Ok(())
    }
}
