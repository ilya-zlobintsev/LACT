mod driver;
pub mod nvapi;

use super::{CommonControllerInfo, FanControlHandle, GpuController};
use crate::{
    bindings::nvidia::NvPhysicalGpuHandle,
    server::{
        gpu_controller::{common::fan_control::FanCurveExt, common::resolve_process_name, NvApi},
        opencl::get_opencl_info,
        vulkan::get_vulkan_info,
    },
};
use amdgpu_sysfs::{gpu_handle::power_profile_mode::PowerProfileModesTable, hw_mon::Temperature};
use anyhow::{anyhow, bail, Context};
use driver::DriverHandle;
use futures::{future::LocalBoxFuture, FutureExt};
use indexmap::IndexMap;
use lact_schema::{
    config::{FanControlSettings, FanCurve, GpuConfig},
    CacheInfo, ClocksInfo, ClocksTable, ClockspeedStats, DeviceInfo, DeviceStats, DeviceType,
    DrmInfo, DrmMemoryInfo, FanControlMode, FanStats, IntelDrmInfo, LinkInfo, NvidiaClockOffset,
    NvidiaClocksTable, PmfwInfo, PowerState, PowerStates, PowerStats, ProcessInfo, ProcessList,
    ProcessType, ProcessUtilizationType, VoltageStats, VramStats,
};
use nvml_wrapper::{
    bitmasks::device::ThrottleReasons,
    enum_wrappers::device::{Clock, PerformanceState, TemperatureSensor, TemperatureThreshold},
    enums::device::{GpuLockedClocksSetting, UsedGpuMemory},
    error::NvmlError,
    Device, Nvml,
};
use std::{
    cell::{Cell, RefCell},
    cmp,
    collections::{btree_map::Entry, BTreeMap, HashMap},
    fmt::Write,
    rc::Rc,
    time::{Duration, Instant},
};
use tokio::{select, sync::Notify, time::sleep};
use tracing::{debug, error, trace, warn};

const SUPPORTED_UTIL_TYPES: &[ProcessUtilizationType] = &[
    ProcessUtilizationType::Graphics,
    ProcessUtilizationType::Memory,
    ProcessUtilizationType::Encode,
    ProcessUtilizationType::Decode,
];

pub struct NvidiaGpuController {
    nvml: Rc<Nvml>,
    nvapi: Rc<Option<NvApi>>,
    common: CommonControllerInfo,
    fan_control_handle: RefCell<Option<FanControlHandle>>,

    driver_handle: Option<DriverHandle>,
    nvapi_handle: Option<NvPhysicalGpuHandle>,
    nvapi_thermals_mask: Option<i32>,

    last_util_timestamp: Cell<Option<u64>>,
    // Store last applied offsets as a workaround when the driver doesn't tell us the current offset
    last_applied_offsets: RefCell<HashMap<Clock, HashMap<PerformanceState, i32>>>,
    last_applied_gpu_locked_clocks: RefCell<Option<(u32, u32)>>,
    last_applied_vram_locked_clocks: RefCell<Option<(u32, u32)>>,
}

impl NvidiaGpuController {
    pub fn new(
        common: CommonControllerInfo,
        nvml: Rc<Nvml>,
        nvapi: Rc<Option<NvApi>>,
    ) -> anyhow::Result<Self> {
        let device = nvml
            .device_by_pci_bus_id(common.pci_slot_name.as_str())
            .with_context(|| {
                format!(
                    "Could not get PCI device '{}' from NVML",
                    common.pci_slot_name
                )
            })?;

        let (nvapi_handle, nvapi_thermals_mask) = match nvapi.as_ref() {
            Some(nvapi) => {
                let bus_id = common.get_slot_info()?.bus;
                let gpu_handle = nvapi
                    .find_matching_gpu(u32::from(bus_id))
                    .inspect_err(|err| error!("Could not get NvAPI GPU handle: {err}"))
                    .ok()
                    .flatten();

                let thermals_mask = gpu_handle.and_then(|handle| unsafe {
                    nvapi
                        .calculate_thermals_mask(handle)
                        .inspect(|mask| {
                            debug!("calculated NvAPI thermals mask {mask:x}");
                        })
                        .inspect_err(|err| {
                            error!("could not calculate NvAPI thermal mask: {err:#}");
                        })
                        .ok()
                });

                (gpu_handle, thermals_mask)
            }
            None => (None, None),
        };
        debug!("initialized NvAPI device handle {nvapi_handle:?}");

        let minor_number = device.minor_number()?;

        let driver_handle = match DriverHandle::open(minor_number) {
            Ok(handle) => {
                debug!("opened Nvidia driver handle");
                Some(handle)
            }
            Err(err) => {
                error!("could not get Nvidia driver handle: {err:#}");
                None
            }
        };

        Ok(Self {
            nvml,
            nvapi,
            common,
            driver_handle,
            nvapi_handle,
            nvapi_thermals_mask,
            last_util_timestamp: Cell::new(None),
            fan_control_handle: RefCell::new(None),
            last_applied_offsets: RefCell::new(HashMap::new()),
            last_applied_gpu_locked_clocks: RefCell::new(None),
            last_applied_vram_locked_clocks: RefCell::new(None),
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
            #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
            let auto_threshold = settings.auto_threshold.unwrap_or(0) as i32;

            let mut manual_mode = true;

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

                if current_temp < auto_threshold {
                    if manual_mode {
                        trace!("temperature below auto threshold, setting fan policy to auto");
                        for fan in 0..fan_count {
                            if let Err(err) = device.set_default_fan_speed(fan) {
                                error!(
                                    "could not set fan speed to auto: {err}, disabling fan control"
                                );
                                break;
                            }
                        }

                        manual_mode = false;
                    } else {
                        trace!("temperature below auto threshold, skipping control");
                    }
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
                manual_mode = true;
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
}

impl GpuController for NvidiaGpuController {
    fn controller_info(&self) -> &CommonControllerInfo {
        &self.common
    }

    fn device_type(&self) -> DeviceType {
        // No clue what happens on Tegra chips
        DeviceType::Dedicated
    }

    fn get_info(&self) -> LocalBoxFuture<'_, DeviceInfo> {
        Box::pin(async move {
            let vulkan_instances = get_vulkan_info(&self.common).await.unwrap_or_else(|err| {
                warn!("could not load vulkan info: {err:#}");
                vec![]
            });
            let device = self.device();
            let driver_handle = self.driver_handle.as_ref();

            DeviceInfo {
                pci_info: Some(self.common.pci_info.clone()),
                vulkan_instances,
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
                                let _ = write!(output, " Gen {gen}");
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
                                let _ = write!(output, " Gen {gen}");
                            }
                            output
                        }),
                },
                opencl_info: get_opencl_info(&self.common),
                drm_info: Some(DrmInfo {
                    device_name: device.name().ok(),
                    pci_revision_id: None,
                    family_name: device.architecture().map(|arch| arch.to_string()).ok(),
                    family_id: None,
                    asic_name: None,
                    chip_class: device.architecture().map(|arch| arch.to_string()).ok(),
                    compute_units: None,
                    streaming_multiprocessors: driver_handle
                        .and_then(|handle| handle.get_sm_count().ok()),
                    cuda_cores: device.num_cores().ok(),
                    vram_type: driver_handle
                        .and_then(|handle| handle.get_ram_type().ok())
                        .map(str::to_owned),
                    vram_clock_ratio: 1.0,
                    isa: None,
                    vram_vendor: driver_handle
                        .and_then(|handle| handle.get_ram_vendor().ok())
                        .map(str::to_owned),
                    vram_bit_width: driver_handle.and_then(|handle| handle.get_bus_width().ok()),
                    vram_max_bw: None,
                    cache_info: driver_handle
                        .and_then(|handle| handle.get_l2_cache_size().ok())
                        .map(|size| CacheInfo::Nvidia { l2: size }),
                    rop_info: driver_handle
                        .as_ref()
                        .and_then(|handle| handle.get_rop_info().ok()),
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
        })
    }

    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::too_many_lines
    )]
    fn get_stats(&self, gpu_config: Option<&GpuConfig>) -> DeviceStats {
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

        let mut voltage = None;

        if let (Some(nvapi), Some(handle)) = (self.nvapi.as_ref(), self.nvapi_handle.as_ref()) {
            unsafe {
                if let Some(mask) = self.nvapi_thermals_mask {
                    if let Ok(thermals) = nvapi.get_thermals(*handle, mask) {
                        if let Some(hotspot) = thermals.hotspot() {
                            temps.insert(
                                "GPU Hotspot".to_owned(),
                                Temperature {
                                    current: Some(hotspot as f32),
                                    crit: None,
                                    crit_hyst: None,
                                },
                            );
                        }

                        if let Some(vram) = thermals.vram() {
                            temps.insert(
                                "VRAM".to_owned(),
                                Temperature {
                                    current: Some(vram as f32),
                                    crit: None,
                                    crit_hyst: None,
                                },
                            );
                        }
                    }
                }

                if let Ok(value) = nvapi.get_voltage(*handle) {
                    voltage = Some(u64::from(value) / 1000);
                }
            }
        }

        let fan_settings = gpu_config.and_then(|config| config.fan_control_settings.as_ref());

        let num_fans = device.num_fans().unwrap_or(0);

        let (pwm_current, speed_current) = if num_fans == 0 {
            (None, None)
        } else {
            let fan_speeds = (0..num_fans)
                .flat_map(|idx| device.fan_speed(idx))
                .collect::<Vec<_>>();

            let pwm_current = if fan_speeds.is_empty() {
                None
            } else {
                let avg_speed: u32 = fan_speeds.iter().sum::<u32>() / fan_speeds.len() as u32;
                Some((f64::from(avg_speed) * 2.55) as u8)
            };

            let fan_speeds_rpm = (0..num_fans)
                .flat_map(|idx| device.fan_speed_rpm(idx))
                .collect::<Vec<_>>();

            let speed_current = if fan_speeds_rpm.is_empty() {
                None
            } else {
                Some(fan_speeds_rpm.iter().sum::<u32>() / fan_speeds_rpm.len() as u32)
            };

            (pwm_current, speed_current)
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

        let fan_range = device.min_max_fan_speed().ok();

        DeviceStats {
            temps,
            fan: FanStats {
                control_enabled: gpu_config.is_some_and(|config| config.fan_control_enabled),
                control_mode: fan_settings.map(|settings| settings.mode),
                static_speed: fan_settings.map(|settings| settings.static_speed),
                curve: fan_settings.map(|settings| settings.curve.0.clone()),
                spindown_delay_ms: fan_settings.and_then(|settings| settings.spindown_delay_ms),
                change_threshold: fan_settings.and_then(|settings| settings.change_threshold),
                auto_threshold: fan_settings.and_then(|settings| settings.auto_threshold),
                temperature_key: None,
                speed_current,
                speed_max: None,
                speed_min: None,
                pwm_current,
                pwm_max: fan_range.map(|(_, max)| (f64::from(max) * 2.55).round() as u32),
                pwm_min: fan_range.map(|(min, _)| (f64::from(min) * 2.55).round() as u32),
                temperature_range: None,
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
            voltage: VoltageStats {
                gpu: voltage,
                northbridge: None,
            },
            performance_level: None,
            core_power_state: active_pstate,
            memory_power_state: active_pstate,
            pcie_power_state: None,
        }
    }

    #[allow(clippy::cast_possible_wrap)]
    fn get_clocks_info(&self, _gpu_config: Option<&GpuConfig>) -> anyhow::Result<ClocksInfo> {
        let device = self.device();

        let mut gpu_offsets = IndexMap::new();
        let mut mem_offsets = IndexMap::new();
        let mut gpu_clock_range = None;
        let mut vram_clock_range = None;

        let supported_pstates = device.supported_performance_states()?;

        let mut clock_types = [
            (Clock::Graphics, &mut gpu_offsets, &mut gpu_clock_range),
            (Clock::Memory, &mut mem_offsets, &mut vram_clock_range),
        ];

        for pstate in supported_pstates.iter().rev() {
            for (clock_type, offsets, clock_range) in &mut clock_types {
                if let Ok(offset) = device.clock_offset(*clock_type, *pstate) {
                    let mut offset = NvidiaClockOffset {
                        current: offset.clock_offset_mhz,
                        min: offset.min_clock_offset_mhz,
                        max: offset.max_clock_offset_mhz,
                    };

                    // On some driver versions, the applied offset values are not reported.
                    // In these scenarios we must store them manually for reporting.
                    if offset.current == 0 {
                        if let Some(applied_offsets) =
                            self.last_applied_offsets.borrow().get(clock_type)
                        {
                            if let Some(applied_offset) = applied_offsets.get(pstate) {
                                offset.current = *applied_offset;
                            }
                        }
                    }

                    offsets.insert(pstate.as_c(), offset);
                }

                // Find min and max clockspeeds from any pstate
                if let Ok((pstate_min, pstate_max)) =
                    device.min_max_clock_of_pstate(*clock_type, *pstate)
                {
                    match clock_range {
                        Some((current_min, current_max)) => {
                            *current_min = cmp::min(*current_min, pstate_min);
                            *current_max = cmp::max(*current_max, pstate_max);
                        }
                        None => {
                            **clock_range = Some((pstate_min, pstate_max));
                        }
                    }
                }
            }
        }

        let table = NvidiaClocksTable {
            gpu_offsets,
            mem_offsets,
            gpu_locked_clocks: *self.last_applied_gpu_locked_clocks.borrow(),
            vram_locked_clocks: *self.last_applied_vram_locked_clocks.borrow(),
            gpu_clock_range,
            vram_clock_range,
        };

        Ok(ClocksInfo {
            table: Some(ClocksTable::Nvidia(table)),
            ..Default::default()
        })
    }

    fn get_power_states(&self, _gpu_config: Option<&GpuConfig>) -> PowerStates {
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

    #[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
    fn apply_config<'a>(&'a self, config: &'a GpuConfig) -> LocalBoxFuture<'a, anyhow::Result<()>> {
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

            self.reset_clocks()?;

            let clocks = &config.clocks_configuration;

            match (clocks.min_core_clock, clocks.max_core_clock) {
                (Some(min), Some(max)) => {
                    debug!("applying GPU locked clocks: {min}..{max}");
                    device
                        .set_gpu_locked_clocks(GpuLockedClocksSetting::Numeric {
                            min_clock_mhz: min as u32,
                            max_clock_mhz: max as u32,
                        })
                        .context("Could not apply GPU locked clocks")?;
                    self.last_applied_gpu_locked_clocks
                        .replace(Some((min as u32, max as u32)));
                }
                (None, None) => (),
                _ => bail!("Min and max GPU clock must be set together"),
            }

            match (clocks.min_memory_clock, clocks.max_memory_clock) {
                (Some(min), Some(max)) => {
                    debug!("applying VRAM locked clocks: {min}..{max}");
                    device
                        .set_mem_locked_clocks(min as u32, max as u32)
                        .context("Could not apply VRAM locked clocks")?;
                    self.last_applied_vram_locked_clocks
                        .replace(Some((min as u32, max as u32)));
                }
                (None, None) => (),
                _ => bail!("Min and max VRAM clock must be set together"),
            }

            for (pstate, offset) in &clocks.gpu_clock_offsets {
                let pstate = PerformanceState::try_from(*pstate)
                    .map_err(|_| anyhow!("Invalid pstate '{pstate}'"))?;
                debug!("applying offset {offset} for GPU pstate {pstate:?}");
                device
                    .set_clock_offset(Clock::Graphics, pstate, *offset)
                    .with_context(|| {
                        format!("Could not set clock offset {offset} for GPU pstate {pstate:?}")
                    })?;

                self.last_applied_offsets
                    .borrow_mut()
                    .entry(Clock::Graphics)
                    .or_default()
                    .insert(pstate, *offset);
            }

            for (pstate, offset) in &clocks.mem_clock_offsets {
                let pstate = PerformanceState::try_from(*pstate)
                    .map_err(|_| anyhow!("Invalid pstate '{pstate}'"))?;
                debug!("applying offset {offset} for VRAM pstate {pstate:?}");
                device
                    .set_clock_offset(Clock::Memory, pstate, *offset)
                    .with_context(|| {
                        format!("Could not set clock offset {offset} for VRAM pstate {pstate:?}")
                    })?;

                self.last_applied_offsets
                    .borrow_mut()
                    .entry(Clock::Memory)
                    .or_default()
                    .insert(pstate, *offset);
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
                        let (min_speed, max_speed) = device
                            .min_max_fan_speed()
                            .context("Could not get fan speed range")?;

                        for point in settings.curve.0.values() {
                            #[allow(clippy::cast_possible_truncation)]
                            if !(min_speed..=max_speed).contains(&((*point * 100.0) as u32)) {
                                bail!("Fan speed {}% outside of the allowed range {min_speed}% to {max_speed}%", point*100.0);
                            }
                        }

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

    fn reset_clocks(&self) -> anyhow::Result<()> {
        let mut device = self.device();

        if let Ok(supported_pstates) = device.supported_performance_states() {
            for pstate in supported_pstates {
                for clock_type in [Clock::Graphics, Clock::Memory] {
                    if let Ok(current_offset) = device.clock_offset(clock_type, pstate) {
                        if current_offset.clock_offset_mhz != 0
                            || self
                                .last_applied_offsets
                                .borrow()
                                .get(&clock_type)
                                .and_then(|applied_offsets| applied_offsets.get(&pstate))
                                .is_some_and(|offset| *offset != 0)
                        {
                            debug!("resetting clock offset for {clock_type:?} pstate {pstate:?}");
                            device
                                .set_clock_offset(clock_type, pstate, 0)
                                .with_context(|| {
                                    format!("Could not reset {clock_type:?} pstate {pstate:?}")
                                })?;
                        }
                    }

                    if let Some(applied_offsets) =
                        self.last_applied_offsets.borrow_mut().get_mut(&clock_type)
                    {
                        applied_offsets.remove(&pstate);
                    }
                }
            }
        }

        if self.last_applied_gpu_locked_clocks.borrow().is_some() {
            device
                .reset_gpu_locked_clocks()
                .context("Could not reset locked GPU clocks")?;
            self.last_applied_gpu_locked_clocks.take();
        }

        if self.last_applied_vram_locked_clocks.borrow().is_some() {
            device
                .reset_mem_locked_clocks()
                .context("Could not reset locked GPU clocks")?;
            self.last_applied_vram_locked_clocks.take();
        }

        Ok(())
    }

    fn cleanup(&self) -> LocalBoxFuture<'_, ()> {
        async {
            if let Some((fan_notify, fan_handle)) = self.fan_control_handle.take() {
                debug!("sending stop notification to old fan control task");
                fan_notify.notify_one();
                fan_handle.await.unwrap();
                debug!("finished controller cleanup");
            }
        }
        .boxed_local()
    }

    fn process_list(&self) -> anyhow::Result<ProcessList> {
        fn map_process(
            process: &nvml_wrapper::struct_wrappers::device::ProcessInfo,
            process_type: ProcessType,
        ) -> ProcessInfo {
            #[allow(clippy::cast_possible_wrap)]
            let (name, args) = resolve_process_name((process.pid as i32).into())
                .unwrap_or_else(|_| ("<Unknown>".to_owned(), String::new()));

            ProcessInfo {
                name,
                args,
                memory_used: match process.used_gpu_memory {
                    UsedGpuMemory::Used(size) => size,
                    UsedGpuMemory::Unavailable => 0,
                },
                types: vec![process_type],
                util: SUPPORTED_UTIL_TYPES.iter().map(|util| (*util, 0)).collect(),
            }
        }

        let device = self.device();

        let mut processes = BTreeMap::new();

        for process in device
            .running_graphics_processes()
            .context("Could not get graphics processes")?
        {
            processes.insert(process.pid, map_process(&process, ProcessType::Graphics));
        }

        for process in device
            .running_compute_processes()
            .context("Could not get compute processes")?
        {
            match processes.entry(process.pid) {
                Entry::Vacant(entry) => {
                    entry.insert(map_process(&process, ProcessType::Compute));
                }
                Entry::Occupied(mut entry) => {
                    entry.get_mut().types.push(ProcessType::Compute);
                }
            }
        }

        match device.process_utilization_stats(self.last_util_timestamp.get()) {
            Ok(stats) => {
                if let Some(stat) = stats.first() {
                    self.last_util_timestamp.set(Some(stat.timestamp));
                }

                for stat in stats {
                    if let Some(info) = processes.get_mut(&stat.pid) {
                        info.util
                            .insert(ProcessUtilizationType::Graphics, stat.sm_util);
                        info.util
                            .insert(ProcessUtilizationType::Memory, stat.mem_util);
                        info.util
                            .insert(ProcessUtilizationType::Encode, stat.enc_util);
                        info.util
                            .insert(ProcessUtilizationType::Decode, stat.dec_util);
                    }
                }
            }
            Err(NvmlError::NotFound) => (),
            Err(err) => {
                error!("could not get process util stats: {err}");
            }
        }
        Ok(ProcessList {
            processes,
            supported_util_types: SUPPORTED_UTIL_TYPES.iter().copied().collect(),
        })
    }
}
