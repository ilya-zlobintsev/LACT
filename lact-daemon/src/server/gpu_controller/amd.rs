use super::{CommonControllerInfo, FanControlHandle, GpuController, VENDOR_AMD};
use crate::server::{
    gpu_controller::common::{
        fan_control::FanCurveExt,
        fdinfo::{self, DrmUtilMap},
    },
    opencl::get_opencl_info,
    vulkan::get_vulkan_info,
};
use amdgpu_sysfs::{
    error::Error,
    gpu_handle::{
        fan_control::FanCurve as PmfwCurve,
        overdrive::{ClocksTable, ClocksTableGen},
        power_profile_mode::PowerProfileModesTable,
        CommitHandle, GpuHandle, PerformanceLevel, PowerLevelKind, PowerLevels,
    },
    hw_mon::{FanControlMethod, HwMon, Temperature},
    sysfs::SysFS,
};
use anyhow::{anyhow, bail, Context};
use futures::{future::LocalBoxFuture, join, FutureExt};
use lact_schema::{
    config::{ClocksConfiguration, FanControlSettings, FanCurve, GpuConfig},
    AmdCacheInstance, CacheInfo, CacheType, ClocksInfo, ClockspeedStats, DeviceFlag, DeviceInfo,
    DeviceStats, DeviceType, DrmInfo, FanControlMode, FanStats, IntelDrmInfo, LinkInfo, PmfwInfo,
    PowerState, PowerStates, PowerStats, ProcessList, ProcessUtilizationType, RopInfo,
    TemperatureEntry, VoltageStats, VramStats,
};
use libdrm_amdgpu_sys::AMDGPU::{GpuMetrics, ThrottlerBit};
use libdrm_amdgpu_sys::{LibDrmAmdgpu, AMDGPU::SENSOR_INFO::SENSOR_TYPE, PCI};
use std::{
    cell::RefCell,
    cmp,
    collections::{HashMap, HashSet, VecDeque},
    path::{Path, PathBuf},
    rc::Rc,
    time::Duration,
};
use std::{collections::BTreeMap, fs, time::Instant};
use tokio::{select, sync::Notify, time::sleep};
use tracing::{debug, error, info, trace, warn};

use {
    lact_schema::DrmMemoryInfo,
    libdrm_amdgpu_sys::AMDGPU::{DeviceHandle as DrmHandle, MetricsInfo, GPU_INFO},
};

/// RDNA3 - minimum family with PMFW
const AMDGPU_FAMILY_GC_11_0_0: u32 = 145;

const FAN_CONTROL_RETRIES: u32 = 10;
const MAX_PSTATE_READ_ATTEMPTS: u32 = 5;
const STEAM_DECK_IDS: [&str; 2] = ["163F", "1435"];
const AMDGPU_IDS_FLAGS_FUSION: u64 = 0x1;
const HSA_CACHE_TYPE_DATA: u32 = 0x0000_0001;
const HSA_CACHE_TYPE_INSTRUCTION: u32 = 0x0000_0002;
const HSA_CACHE_TYPE_CPU: u32 = 0x0000_0004;
// const HSA_CACHE_TYPE_HSACU: u32 = 0x0000_0008;

const DRM_VRAM_KEYS: &[&str] = &["drm-memory-vram"];
const DRM_ENGINES: &[(&str, ProcessUtilizationType)] = &[
    ("gfx", ProcessUtilizationType::Graphics),
    ("compute", ProcessUtilizationType::Compute),
    ("dec", ProcessUtilizationType::Decode),
    ("enc", ProcessUtilizationType::Encode),
    ("enc_1", ProcessUtilizationType::Encode),
];

pub struct AmdGpuController {
    handle: GpuHandle,
    drm_handle: Option<DrmHandle>,
    common: CommonControllerInfo,
    fan_control_handle: RefCell<Option<FanControlHandle>>,
    last_drm_util: RefCell<Option<DrmUtilMap>>,
}

impl AmdGpuController {
    #[allow(unused_variables)]
    pub fn new_from_path(
        common: CommonControllerInfo,
        libdrm_amdgpu: Option<&'static LibDrmAmdgpu>,
    ) -> anyhow::Result<Self> {
        let handle = GpuHandle::new_from_path(common.sysfs_path.clone())
            .map_err(|error| anyhow!("failed to initialize gpu handle: {error}"))?;

        #[allow(unused_mut)]
        let mut drm_handle = None;
        #[cfg(not(test))]
        if let Some(libdrm_amdgpu) = libdrm_amdgpu {
            if handle.get_driver() == "amdgpu" {
                drm_handle = Some(
                    get_drm_handle(&common, libdrm_amdgpu)
                        .context("Could not get AMD DRM handle")?,
                );
            }
        }

        Ok(Self {
            handle,
            drm_handle,
            common,
            fan_control_handle: RefCell::new(None),
            last_drm_util: RefCell::new(None),
        })
    }

    fn hw_mon_and_then<U>(&self, f: fn(&HwMon) -> Result<U, Error>) -> Option<U> {
        self.handle.hw_monitors.first().and_then(|mon| f(mon).ok())
    }

    async fn set_static_fan_control(&self, static_speed: f32) -> anyhow::Result<Vec<CommitHandle>> {
        // Stop existing task to set static speed
        self.stop_fan_control(false).await?;

        let mut commit_handles = Vec::new();

        // Use PMFW curve functionality for static speed when it is available
        if let Ok(current_curve) = self.handle.get_fan_curve() {
            if let Ok(true) = self.handle.get_fan_zero_rpm_enable() {
                match self.handle.set_fan_zero_rpm_enable(false) {
                    Ok(zero_rpm_commit) => {
                        commit_handles.push(zero_rpm_commit);
                    }
                    Err(err) => {
                        error!("could not disable zero RPM mode for static fan control: {err}");
                    }
                }
            }

            let allowed_ranges = current_curve.allowed_ranges.clone().ok_or_else(|| {
                anyhow!("The GPU does not allow setting custom fan values (is overdrive enabled?)")
            })?;
            let min_temperature = allowed_ranges.temperature_range.start();
            let max_temperature = allowed_ranges.temperature_range.end();

            #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
            let custom_pwm = (f32::from(*allowed_ranges.speed_range.end()) * static_speed) as u8;
            let static_pwm = cmp::max(*allowed_ranges.speed_range.start(), custom_pwm);

            let mut points = vec![(*min_temperature, static_pwm)];
            for _ in 1..current_curve.points.len() {
                points.push((*max_temperature, static_pwm));
            }

            let new_curve = PmfwCurve {
                points: points.into_boxed_slice(),
                allowed_ranges: Some(allowed_ranges),
            };

            debug!("setting static curve {new_curve:?}");

            let curve_commit = self
                .handle
                .set_fan_curve(&new_curve)
                .context("Could not set fan curve")?;
            commit_handles.push(curve_commit);

            Ok(commit_handles)
        } else {
            let hw_mon = self
                .handle
                .hw_monitors
                .first()
                .cloned()
                .context("This GPU has no monitor")?;

            hw_mon
                .set_fan_control_method(FanControlMethod::Manual)
                .context("Could not set fan control method")?;

            #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
            let static_pwm = (f32::from(u8::MAX) * static_speed) as u8;

            hw_mon
                .set_fan_pwm(static_pwm)
                .context("could not set fan speed")?;

            debug!("set fan speed to {}", static_speed);

            Ok(vec![])
        }
    }

    async fn start_curve_fan_control(
        &self,
        curve: FanCurve,
        settings: FanControlSettings,
    ) -> anyhow::Result<Option<CommitHandle>> {
        // Use the PMFW curve functionality when it is available
        // Otherwise, fall back to manual fan control via a task
        if let Ok(current_curve) = self.handle.get_fan_curve() {
            let new_curve = curve
                .into_pmfw_curve(current_curve.clone())
                .context("Invalid fan curve")?;

            debug!("setting pmfw curve {new_curve:?}");

            let commit_handle = self
                .handle
                .set_fan_curve(&new_curve)
                .context("Could not set fan curve")?;

            Ok(Some(commit_handle))
        } else {
            self.start_curve_fan_control_task(curve, settings).await?;
            Ok(None)
        }
    }

    async fn start_curve_fan_control_task(
        &self,
        curve: FanCurve,
        settings: FanControlSettings,
    ) -> anyhow::Result<()> {
        // Stop existing task to re-apply new curve
        self.stop_fan_control(false).await?;

        let hw_mon = self
            .handle
            .hw_monitors
            .first()
            .cloned()
            .context("This GPU has no monitor")?;

        let temps = hw_mon.get_temps();
        match temps.len() {
            0 => return Err(anyhow!("GPU has no temperature reporting")),
            1 => {
                warn!("GPU has only one temperature sensor, 'temperature_key' setting will be ignored");
            }
            _ => {
                if !temps.contains_key(&settings.temperature_key) {
                    return Err(anyhow!(
                        "Sensor with name {} not found, available sensors: {}",
                        settings.temperature_key,
                        temps
                            .keys()
                            .map(String::as_str)
                            .collect::<Vec<&str>>()
                            .join(",")
                    ));
                }
            }
        }

        hw_mon
            .set_fan_control_method(FanControlMethod::Manual)
            .context("Could not set fan control method")?;

        let mut notify_guard = self
            .fan_control_handle
            .try_borrow_mut()
            .map_err(|err| anyhow!("Lock error: {err}"))?;

        let notify = Rc::new(Notify::new());
        let task_notify = notify.clone();

        debug!("spawning new fan control task");
        let handle = tokio::task::spawn_local(async move {
            let mut last_pwm = (None, Instant::now());
            let mut last_temp = 0.0;

            // If the fan speed could was able to be set at least once
            let mut control_available = false;

            let temp_key = settings.temperature_key.clone();
            let interval = Duration::from_millis(settings.interval_ms);
            let spindown_delay = Duration::from_millis(settings.spindown_delay_ms.unwrap_or(0));
            #[allow(clippy::cast_precision_loss)]
            let change_threshold = settings.change_threshold.unwrap_or(0) as f32;

            let mut retries = 0;

            loop {
                select! {
                    () = sleep(interval) => (),
                    () = task_notify.notified() => break,
                }

                let mut temps = hw_mon.get_temps();
                let temp = if temps.len() == 1 {
                    temps.into_values().next().unwrap()
                } else if let Some(value) = temps.remove(&temp_key) {
                    value
                } else {
                    retries += 1;

                    if retries == FAN_CONTROL_RETRIES {
                        error!("could not get temperature sensor {temp_key}, exiting fan control (reached max attempts)");
                        break;
                    }
                    error!("could not get temperature sensor {temp_key} from {} sensors (assuming error is temporary, attempt {retries}/{FAN_CONTROL_RETRIES})", temps.len());
                    continue;
                };

                let current_temp = temp.current.expect("Missing temp");

                if (last_temp - current_temp).abs() < change_threshold {
                    trace!("temperature changed from {last_temp}°C to {current_temp}°C, which is less than the {change_threshold}°C threshold, skipping speed adjustment");
                    continue;
                }

                let target_pwm = curve.pwm_at_temp(temp);
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

                match hw_mon.set_fan_pwm(target_pwm) {
                    Ok(()) => control_available = true,
                    Err(err) => {
                        error!("could not set fan speed: {err}");
                        if control_available {
                            retries += 1;

                            if retries == FAN_CONTROL_RETRIES {
                                error!("could not set fan speed after {retries} attempts, exiting fan control (reached max attempts)");
                                break;
                            }

                            info!("fan control was previously available, assuming the error is temporary (attempt {retries}/{FAN_CONTROL_RETRIES})");

                            if !matches!(
                                hw_mon.get_fan_control_method(),
                                Ok(FanControlMethod::Manual),
                            ) {
                                info!("fan control method was changed externally, setting back to manual");
                                if let Err(err) =
                                    hw_mon.set_fan_control_method(FanControlMethod::Manual)
                                {
                                    error!("could not set fan control back to manual: {err}");
                                    break;
                                }
                            }
                        } else {
                            info!("disabling fan control");
                            break;
                        }
                    }
                }
                retries = 0;
            }
            debug!("exited fan control task");

            if let Err(err) = hw_mon.set_fan_control_method(FanControlMethod::Auto) {
                error!("could not reset fan control back to auto: {err}");
            }
        });

        *notify_guard = Some((notify, handle));

        debug!(
            "started fan control with interval {}ms",
            settings.interval_ms
        );

        Ok(())
    }

    async fn stop_fan_control(&self, reset_mode: bool) -> anyhow::Result<()> {
        let maybe_notify = self
            .fan_control_handle
            .try_borrow_mut()
            .map_err(|err| anyhow!("Lock error: {err}"))?
            .take();
        if let Some((notify, handle)) = maybe_notify {
            notify.notify_one();
            handle.await?;
        }

        if reset_mode {
            if self.handle.get_fan_curve().is_ok() {
                if let Err(err) = self.handle.reset_fan_curve() {
                    warn!("could not reset fan curve: {err:#}");
                }
            }

            if let Some(hw_mon) = self.handle.hw_monitors.first().cloned() {
                if let Ok(current_control) = hw_mon.get_fan_control_method() {
                    if !matches!(current_control, FanControlMethod::Auto) {
                        hw_mon
                            .set_fan_control_method(FanControlMethod::Auto)
                            .context("Could not set fan control back to automatic")?;
                    }
                }
            }
        }

        Ok(())
    }

    fn get_power_states_kind(
        &self,
        gpu_config: Option<&GpuConfig>,
        kind: PowerLevelKind,
        attempt: u32,
    ) -> Vec<PowerState> {
        let enabled_states = gpu_config.and_then(|gpu| gpu.power_states.get(&kind));
        let levels = self
            .handle
            .get_clock_levels(kind)
            .unwrap_or_else(|_| PowerLevels {
                levels: Vec::new(),
                active: None,
            })
            .levels;

        if attempt < MAX_PSTATE_READ_ATTEMPTS
            && levels.iter().any(|value| *value >= u64::from(u16::MAX))
        {
            debug!("GPU reported nonsensical p-state value, retrying");
            return self.get_power_states_kind(gpu_config, kind, attempt + 1);
        }

        levels
            .into_iter()
            .enumerate()
            .map(|(i, value)| {
                let i = u8::try_from(i).unwrap();
                let enabled = enabled_states.is_none_or(|enabled| enabled.contains(&i));
                PowerState {
                    enabled,
                    min_value: None,
                    value,
                    index: Some(i),
                }
            })
            .collect()
    }

    fn first_hw_mon(&self) -> anyhow::Result<&HwMon> {
        self.handle
            .hw_monitors
            .first()
            .context("GPU has no hardware monitor")
    }

    fn get_clockspeed(&self, metrics: Option<&GpuMetrics>) -> ClockspeedStats {
        let vram_clockspeed = self
            .drm_handle
            .as_ref()
            .and_then(|handle| handle.sensor_info(SENSOR_TYPE::GFX_MCLK).ok())
            .map(u64::from)
            .or_else(|| self.hw_mon_and_then(HwMon::get_vram_clockspeed));

        let mut sensors = HashMap::new();

        let mut target_gpu_clockspeed = None;

        if let Some(metrics) = metrics {
            for (label, value) in [
                ("DCLK", metrics.get_current_dclk()),
                ("DCLK1", metrics.get_current_dclk1()),
                ("FCLK", metrics.get_current_fclk()),
                ("VCLK", metrics.get_current_vclk()),
                ("VCLK1", metrics.get_current_vclk()),
                ("UCLK", metrics.get_current_uclk()),
                ("SOCCLK", metrics.get_current_socclk()),
            ] {
                if let Some(value) = value {
                    if value != 0 && value != u16::MAX {
                        sensors.insert(label.to_owned(), u64::from(value));
                    }
                }
            }

            target_gpu_clockspeed = metrics
                .get_current_gfxclk()
                .filter(|value| *value != u16::MAX)
                .map(u64::from);
        }

        ClockspeedStats {
            gpu_clockspeed: self.hw_mon_and_then(HwMon::get_gpu_clockspeed),
            target_gpu_clockspeed,
            vram_clockspeed,
            sensors,
        }
    }

    fn get_full_vbios_version(&self) -> Option<String> {
        if let Some(drm_handle) = &self.drm_handle {
            if let Ok(vbios_info) = drm_handle.get_vbios_info() {
                return Some(format!("{} [{}]", vbios_info.ver, vbios_info.date));
            }
        }

        self.handle.get_vbios_version().ok()
    }

    fn get_drm_info(&self) -> Option<DrmInfo> {
        use libdrm_amdgpu_sys::AMDGPU::VRAM_TYPE;

        let cache_info = self
            .kfd_node_path()
            .inspect(|path| debug!("found KFD node path at '{}'", path.display()))
            .and_then(|path| match read_cache_info_from_kfd(&path) {
                Ok(info) => Some(info),
                Err(err) => {
                    warn!("could not read cache info from kfd sysfs: {err:#}");
                    None
                }
            });

        trace!("Reading DRM info");
        let drm_handle = self.drm_handle.as_ref();

        let drm_memory_info =
            drm_handle
                .and_then(|handle| handle.memory_info().ok())
                .map(|memory_info| DrmMemoryInfo {
                    resizeable_bar: Some(memory_info.check_resizable_bar()),
                    cpu_accessible_used: memory_info.cpu_accessible_vram.heap_usage,
                    cpu_accessible_total: memory_info.cpu_accessible_vram.total_heap_size,
                });

        match drm_handle {
            Some(handle) => handle.device_info().ok().map(|drm_info| DrmInfo {
                device_name: drm_info.find_device_name(),
                pci_revision_id: Some(drm_info.pci_rev_id()),
                family_name: Some(drm_info.get_family_name().to_string()),
                family_id: Some(drm_info.family_id()),
                asic_name: Some(drm_info.get_asic_name().to_string()),
                chip_class: Some(drm_info.get_chip_class().to_string()),
                compute_units: Some(drm_info.cu_active_number),
                isa: drm_info
                    .get_gfx_target_version()
                    .map(|version| version.to_string()),
                streaming_multiprocessors: None,
                cuda_cores: None,
                vram_type: Some(drm_info.get_vram_type().to_string()),
                vram_vendor: self.handle.get_vram_vendor().ok(),
                vram_clock_ratio: match drm_info.get_vram_type() {
                    VRAM_TYPE::GDDR6 => 2.0,
                    _ => 1.0,
                },
                vram_bit_width: Some(drm_info.vram_bit_width),
                vram_max_bw: Some(drm_info.peak_memory_bw_gb().to_string()),
                cache_info,
                memory_info: drm_memory_info,
                rop_info: Some(RopInfo {
                    unit_count: drm_info.rb_pipes(),
                    operations_factor: if drm_info.get_asic_name().rbplus_allowed() {
                        8
                    } else {
                        4
                    },
                    operations_count: drm_info.calc_rop_count(),
                }),
                intel: IntelDrmInfo::default(),
            }),
            None => Some(DrmInfo {
                cache_info,
                vram_clock_ratio: 1.0,
                ..Default::default()
            }),
        }
    }

    fn get_link_info(&self) -> LinkInfo {
        #[cfg(not(test))]
        let gpu_pcie_port_bus = self
            .handle
            .get_pci_slot_name()
            .and_then(|name| name.parse::<PCI::BUS_INFO>().ok())
            .map(|bus_info| bus_info.get_gpu_pcie_port_bus());
        #[cfg(test)]
        let gpu_pcie_port_bus: Option<PCI::BUS_INFO> = None;
        let current_link = gpu_pcie_port_bus.and_then(|bus| bus.get_current_link_info());
        let max_pcie_link = gpu_pcie_port_bus.and_then(|bus| bus.get_max_link_info());
        let max_system_link = gpu_pcie_port_bus.and_then(|bus| bus.get_max_system_link());

        let max_link = if let (Some(max_pcie_link), Some(max_system_link)) =
            (max_pcie_link, max_system_link)
        {
            Some(PCI::LINK {
                r#gen: cmp::min(max_pcie_link.r#gen, max_system_link.r#gen),
                width: cmp::min(max_pcie_link.width, max_system_link.width),
            })
        } else {
            None
        };

        LinkInfo {
            current_width: current_link
                .map(|link| link.width.to_string())
                .or_else(|| self.handle.get_current_link_width().ok()),
            current_speed: current_link
                .map(|link| format!("Gen {}", link.gen))
                .or_else(|| self.handle.get_current_link_speed().ok()),
            max_width: max_link
                .map(|link| link.width.to_string())
                .or_else(|| self.handle.get_max_link_width().ok()),
            max_speed: max_link
                .map(|link| format!("Gen {}", link.gen))
                .or_else(|| self.handle.get_max_link_speed().ok()),
        }
    }

    fn get_throttle_info(&self) -> Option<BTreeMap<String, Vec<String>>> {
        use libdrm_amdgpu_sys::AMDGPU::ThrottlerType;

        self.drm_handle
            .as_ref()
            .and_then(|drm_handle| drm_handle.get_gpu_metrics().ok())
            .and_then(|metrics| metrics.get_throttle_status_info())
            .map(|throttle| {
                let mut grouped_bits: HashMap<ThrottlerType, HashSet<u8>> = HashMap::new();

                for bit in throttle.get_all_throttler() {
                    let throttle_type = ThrottlerType::from(bit);
                    grouped_bits
                        .entry(throttle_type)
                        .or_default()
                        .insert(bit as u8);
                }

                grouped_bits
                    .into_iter()
                    .map(|(throttle_type, bits)| {
                        let mut names: Vec<String> = bits
                            .into_iter()
                            .map(|bit| ThrottlerBit::from(bit).to_string())
                            .collect();
                        names.sort_unstable();
                        (throttle_type.to_string(), names)
                    })
                    .collect()
            })
    }

    fn debugfs_path(&self) -> Option<PathBuf> {
        let slot_id = self.handle.get_pci_slot_name()?;
        let name_search_term = format!("dev={slot_id}");

        for entry in fs::read_dir("/sys/kernel/debug/dri").ok()?.flatten() {
            let debugfs_path = entry.path();
            let name_file_path = debugfs_path.join("name");
            if name_file_path.exists() {
                if let Ok(contents) = fs::read_to_string(&name_file_path) {
                    if contents.contains(&name_search_term) {
                        return Some(debugfs_path);
                    }
                }
            }
        }

        None
    }

    fn is_steam_deck(&self) -> bool {
        self.common.pci_info.device_pci_info.vendor_id == VENDOR_AMD
            && STEAM_DECK_IDS.contains(&self.common.pci_info.device_pci_info.model_id.as_str())
    }

    #[cfg(not(test))]
    fn kfd_node_path(&self) -> Option<PathBuf> {
        self.drm_handle
            .as_ref()
            .and_then(|handle| handle.get_pci_bus_info().ok())
            .and_then(|bus_info| bus_info.get_drm_render_path().ok())
            .and_then(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .map(str::to_owned)
            })
            .and_then(|name| name.strip_prefix("renderD").map(str::to_owned))
            .and_then(|render_minor| {
                let nodes_dir = fs::read_dir("/sys/class/kfd/kfd/topology/nodes").ok()?;
                nodes_dir.into_iter().flatten().find_map(|entry| {
                    fs::read_to_string(entry.path().join("properties"))
                        .ok()
                        .filter(|node_properties| {
                            node_properties
                                .lines()
                                .filter_map(|line| line.split_once(' '))
                                .any(|(key, value)| {
                                    key == "drm_render_minor" && value == render_minor
                                })
                        })
                        .map(|_| entry.path())
                })
            })
    }

    #[cfg(test)]
    fn kfd_node_path(&self) -> Option<PathBuf> {
        self.handle
            .get_path()
            .parent()
            .and_then(|path| path.parent())
            .map(|path| path.join("kfd/topology/nodes/1"))
    }
}

impl GpuController for AmdGpuController {
    fn controller_info(&self) -> &CommonControllerInfo {
        &self.common
    }

    fn device_type(&self) -> DeviceType {
        self.drm_handle
            .as_ref()
            .and_then(|drm| drm.device_info().ok())
            .map_or(DeviceType::Dedicated, |info| {
                if (info.ids_flags & AMDGPU_IDS_FLAGS_FUSION) > 0 {
                    DeviceType::Integrated
                } else {
                    DeviceType::Dedicated
                }
            })
    }

    fn get_info(&self, unique_vendor: bool) -> LocalBoxFuture<'_, DeviceInfo> {
        Box::pin(async move {
            let (vulkan_result, opencl_instances) = join!(
                get_vulkan_info(&self.common),
                get_opencl_info(&self.common, unique_vendor)
            );
            let vulkan_instances = vulkan_result.unwrap_or_else(|err| {
                warn!("could not load vulkan info: {err:#}");
                vec![]
            });

            let pci_info = Some(self.common.pci_info.clone());
            let driver = self.handle.get_driver().to_owned();
            let vbios_version = self.get_full_vbios_version();
            let link_info = self.get_link_info();
            let drm_info = self.get_drm_info();

            let mut flags = vec![DeviceFlag::DumpableVBios];

            if drm_info
                .as_ref()
                .and_then(|info| info.family_id)
                .is_some_and(|family| family >= AMDGPU_FAMILY_GC_11_0_0)
            {
                flags.push(DeviceFlag::HasPmfw);
            }

            // Custom fan control can only be used if:
            // - There is a fan with a readable speed
            // - The device does not use PMFW (older than RDNA3) OR has a readable PMFW fan curve (with overdrive on RDNA3+)
            if self
                .hw_mon_and_then(HwMon::get_fan_current)
                .or_else(|| self.hw_mon_and_then(HwMon::get_fan_pwm).map(u32::from))
                .or_else(|| {
                    let metrics = GpuMetrics::get_from_sysfs_path(self.handle.get_path()).ok();
                    metrics
                        .as_ref()
                        .and_then(MetricsInfo::get_current_fan_speed)
                        .map(u32::from)
                })
                .is_some()
                && (!flags.contains(&DeviceFlag::HasPmfw) || self.handle.get_fan_curve().is_ok())
            {
                flags.push(DeviceFlag::ConfigurableFanControl);
            }

            DeviceInfo {
                pci_info,
                vulkan_instances,
                driver,
                vbios_version,
                link_info,
                opencl_instances,
                drm_info,
                flags,
            }
        })
    }

    fn friendly_name(&self) -> Option<String> {
        self.drm_handle
            .as_ref()
            .and_then(|handle| handle.device_info().ok())
            .and_then(|info| info.find_device_name())
            .or_else(|| self.common.pci_info.device_pci_info.model.clone())
    }

    #[allow(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::too_many_lines
    )]
    fn get_stats(&self, gpu_config: Option<&GpuConfig>) -> DeviceStats {
        let metrics = GpuMetrics::get_from_sysfs_path(self.handle.get_path()).ok();
        let metrics = metrics.as_ref();

        let pmfw_curve = self.handle.get_fan_curve().ok();
        let pmfw_curve_range = pmfw_curve
            .as_ref()
            .and_then(|curve| curve.allowed_ranges.as_ref())
            .map(|range| &range.speed_range);

        let pwm_max = pmfw_curve_range
            .map(|range| *range.end())
            .map(|percent| (f64::from(percent) * 2.55) as u32)
            .or_else(|| self.hw_mon_and_then(HwMon::get_fan_max_pwm).map(u32::from));
        let pwm_min = pmfw_curve_range
            .map(|range| *range.start())
            .map(|percent| (f64::from(percent) * 2.55) as u32)
            .or_else(|| self.hw_mon_and_then(HwMon::get_fan_min_pwm).map(u32::from));

        let mut temps: HashMap<String, TemperatureEntry> = HashMap::new();

        if let Ok(hwmon) = self.first_hw_mon() {
            temps = hwmon
                .get_temps()
                .into_iter()
                .map(|(name, value)| {
                    let entry = TemperatureEntry {
                        value,
                        display_only: false,
                    };
                    (name, entry)
                })
                .collect();
        }

        let mut power_sensors = HashMap::new();

        if let Some(metrics) = metrics {
            let extra_temp_sensors = [
                ("soc", metrics.get_temperature_soc().map(|temp| temp / 100)),
                ("vrgfx", metrics.get_temperature_vrgfx()),
                ("vrmem", metrics.get_temperature_vrmem()),
                ("vrsoc", metrics.get_temperature_vrsoc()),
                (
                    "skin",
                    metrics.get_temperature_skin().map(|temp| temp / 100),
                ),
            ];

            for (label, value) in extra_temp_sensors {
                if let Some(value) = value {
                    if value != 0 && value != u16::MAX {
                        temps.insert(
                            label.to_owned(),
                            TemperatureEntry {
                                value: Temperature {
                                    current: Some(f32::from(value)),
                                    crit: None,
                                    crit_hyst: None,
                                },
                                display_only: true,
                            },
                        );
                    }
                }
            }

            let extra_power_sensors = [
                ("apu", metrics.get_average_apu_power()),
                ("core", metrics.get_average_all_core_power()),
                ("gfx", metrics.get_average_gfx_power_u32()),
            ]
            .into_iter()
            .filter(|(_, value)| value.is_some_and(|value| value != 0 && value != u32::MAX))
            .chain(
                [
                    ("soc", metrics.get_average_soc_power()),
                    ("cpu", metrics.get_average_cpu_power()),
                ]
                .into_iter()
                .filter(|(_, value)| value.is_some_and(|value| value != 0 && value != u16::MAX))
                .map(|(label, value)| (label, value.map(u32::from))),
            );

            for (label, value) in extra_power_sensors {
                if let Some(value) = value {
                    power_sensors.insert(label.to_owned(), f64::from(value) / 1000.0);
                }
            }
        }

        let mut voltages = HashMap::new();

        for (label, value) in [
            (
                "northbridge",
                self.hw_mon_and_then(HwMon::get_northbridge_voltage),
            ),
            (
                "mem",
                metrics
                    .and_then(MetricsInfo::get_voltage_mem)
                    .map(u64::from),
            ),
            (
                "soc",
                metrics
                    .and_then(MetricsInfo::get_voltage_soc)
                    .map(u64::from),
            ),
        ] {
            if let Some(value) = value {
                voltages.insert(label.to_owned(), value);
            }
        }

        let fan_settings = gpu_config.and_then(|config| config.fan_control_settings.as_ref());
        DeviceStats {
            fan: FanStats {
                control_enabled: gpu_config.is_some_and(|config| config.fan_control_enabled),
                control_mode: fan_settings.map(|settings| settings.mode),
                static_speed: fan_settings.map(|settings| settings.static_speed),
                curve: fan_settings.map(|settings| settings.curve.0.clone()),
                spindown_delay_ms: fan_settings.and_then(|settings| settings.spindown_delay_ms),
                change_threshold: fan_settings.and_then(|settings| settings.change_threshold),
                temperature_key: fan_settings.map(|settings| settings.temperature_key.clone()),
                auto_threshold: None,
                speed_current: self.hw_mon_and_then(HwMon::get_fan_current).or_else(|| {
                    metrics
                        .and_then(MetricsInfo::get_current_fan_speed)
                        .map(u32::from)
                }),
                speed_max: self.hw_mon_and_then(HwMon::get_fan_max),
                speed_min: self.hw_mon_and_then(HwMon::get_fan_min),
                pwm_current: self.hw_mon_and_then(HwMon::get_fan_pwm).or_else(|| {
                    metrics
                        .and_then(MetricsInfo::get_fan_pwm)
                        .and_then(|pwm| u8::try_from(pwm).ok())
                }),
                pwm_max,
                pwm_min,
                temperature_range: pmfw_curve
                    .as_ref()
                    .and_then(|curve| curve.allowed_ranges.as_ref())
                    .map(|range| {
                        (
                            *range.temperature_range.start(),
                            *range.temperature_range.end(),
                        )
                    }),
                pmfw_info: PmfwInfo {
                    acoustic_limit: self.handle.get_fan_acoustic_limit().ok(),
                    acoustic_target: self.handle.get_fan_acoustic_target().ok(),
                    target_temp: self.handle.get_fan_target_temperature().ok(),
                    minimum_pwm: self.handle.get_fan_minimum_pwm().ok(),
                    zero_rpm_enable: self.handle.get_fan_zero_rpm_enable().ok(),
                    zero_rpm_temperature: self.handle.get_fan_zero_rpm_stop_temperature().ok(),
                },
            },
            clockspeed: self.get_clockspeed(metrics),
            voltage: VoltageStats {
                gpu: self.hw_mon_and_then(HwMon::get_gpu_voltage),
                sensors: voltages,
            },
            vram: VramStats {
                total: self.handle.get_total_vram().ok(),
                used: self.handle.get_used_vram().ok(),
            },
            power: PowerStats {
                average: self.hw_mon_and_then(HwMon::get_power_average),
                current: self.hw_mon_and_then(HwMon::get_power_input),
                cap_current: self.hw_mon_and_then(HwMon::get_power_cap),
                cap_max: self.hw_mon_and_then(HwMon::get_power_cap_max),
                cap_min: self.hw_mon_and_then(HwMon::get_power_cap_min),
                cap_default: self.hw_mon_and_then(HwMon::get_power_cap_default),
                sensors: power_sensors,
            },
            temps,
            busy_percent: self.handle.get_busy_percent().ok(),
            performance_level: self.handle.get_power_force_performance_level().ok(),
            core_power_state: self
                .handle
                .get_core_clock_levels()
                .ok()
                .and_then(|levels| levels.active),
            memory_power_state: self
                .handle
                .get_memory_clock_levels()
                .ok()
                .and_then(|levels| levels.active),
            pcie_power_state: self
                .handle
                .get_pcie_clock_levels()
                .ok()
                .and_then(|levels| levels.active),
            throttle_info: self.get_throttle_info(),
        }
    }

    fn get_clocks_info(&self, _gpu_config: Option<&GpuConfig>) -> anyhow::Result<ClocksInfo> {
        let clocks_table = self
            .handle
            .get_clocks_table()
            .context("Clocks table not available")?;

        Ok(clocks_table.into())
    }

    fn get_power_states(&self, gpu_config: Option<&GpuConfig>) -> PowerStates {
        let core = self.get_power_states_kind(gpu_config, PowerLevelKind::CoreClock, 0);
        let vram = self.get_power_states_kind(gpu_config, PowerLevelKind::MemoryClock, 0);
        PowerStates { core, vram }
    }

    fn get_power_profile_modes(&self) -> anyhow::Result<PowerProfileModesTable> {
        Ok(self.handle.get_power_profile_modes()?)
    }

    fn reset_pmfw_settings(&self) {
        let handle = &self.handle;
        if self.handle.get_fan_target_temperature().is_ok() {
            if let Err(err) = handle.reset_fan_target_temperature() {
                warn!("Could not reset target temperature: {err:#}");
            }
        }
        if self.handle.get_fan_acoustic_target().is_ok() {
            if let Err(err) = handle.reset_fan_acoustic_target() {
                warn!("Could not reset acoustic target: {err:#}");
            }
        }
        if self.handle.get_fan_acoustic_limit().is_ok() {
            if let Err(err) = handle.reset_fan_acoustic_limit() {
                warn!("Could not reset acoustic limit: {err:#}");
            }
        }
        if self.handle.get_fan_minimum_pwm().is_ok() {
            if let Err(err) = handle.reset_fan_minimum_pwm() {
                warn!("Could not reset minimum pwm: {err:#}");
            }
        }
    }

    fn vbios_dump(&self) -> anyhow::Result<Vec<u8>> {
        let debugfs = self.debugfs_path().context("DebugFS not found")?;
        fs::read(debugfs.join("amdgpu_vbios")).context("Could not read VBIOS file")
    }

    #[allow(clippy::too_many_lines)]
    fn apply_config<'a>(&'a self, config: &'a GpuConfig) -> LocalBoxFuture<'a, anyhow::Result<()>> {
        Box::pin(async {
            let mut commit_handles = VecDeque::new();

            // Reset the clocks table in case the settings get reverted back to not having a clocks value configured
            self.handle.reset_clocks_table().ok();

            if !config.fan_control_enabled {
                self.stop_fan_control(true)
                    .await
                    .context("Failed to stop fan control")?;
            }

            if let Some(PerformanceLevel::High | PerformanceLevel::Low) = config.performance_level {
                // Reset to auto first
                self.handle
                    .set_power_force_performance_level(PerformanceLevel::Auto)
                    .context("Failed to set power performance level")?;
            }

            if self.is_steam_deck() {
                // Van Gogh/Sephiroth only allow clock settings to be used with manual performance mode
                self.handle
                    .set_power_force_performance_level(PerformanceLevel::Manual)
                    .ok();
            }

            if config.is_core_clocks_used() {
                match self.handle.get_clocks_table() {
                    Ok(original_table) => {
                        let mut table = original_table.clone();
                        apply_clocks_config_to_table(&config.clocks_configuration, &mut table)
                            .context("Failed to apply clocks configuration to table")?;

                        debug!(
                            "writing clocks commands: {:#?}",
                            table
                                .get_commands(&original_table)
                                .context("Failed to get table commands")?
                        );

                        let handle = self
                            .handle
                            .set_clocks_table(&table)
                            .context("Could not write clocks table")
                            .with_context(|| {
                                format!(
                                    "Clocks table commands: {:?}",
                                    table.get_commands(&original_table)
                                )
                            })?;
                        commit_handles.push_back(handle);
                    }
                    Err(err) => {
                        error!("custom clock settings are present but will be ignored, could not get clocks table: {err}");
                    }
                }
            }

            let mut deferred_performance_level = None;
            match self.handle.get_power_force_performance_level() {
                Ok(_) => {
                    let performance_level =
                        config.performance_level.unwrap_or(PerformanceLevel::Auto);

                    match performance_level {
                        PerformanceLevel::Auto | PerformanceLevel::Manual => {
                            self.handle
                                .set_power_force_performance_level(performance_level)
                                .context("Failed to set power performance level")?;
                        }
                        PerformanceLevel::High | PerformanceLevel::Low => {
                            deferred_performance_level = Some(performance_level);
                        }
                    }
                }
                Err(err) => {
                    error!("could not get current performance level: {err}");
                }
            }

            if let Some(mode_index) = config.power_profile_mode_index {
                if config.performance_level != Some(PerformanceLevel::Manual) {
                    return Err(anyhow!(
                        "Performance level has to be set to `manual` to use power profile modes"
                    ));
                }

                if config.custom_power_profile_mode_hueristics.is_empty() {
                    self.handle
                        .set_active_power_profile_mode(mode_index)
                        .context("Failed to set active power profile mode")?;
                } else {
                    self.handle
                        .set_custom_power_profile_mode_heuristics(
                            &config.custom_power_profile_mode_hueristics,
                        )
                        .context("Failed to set custom power profile mode heuristics")?;
                }
            }

            if config.fan_control_enabled {
                if let Some(ref settings) = config.fan_control_settings {
                    match settings.mode {
                        lact_schema::FanControlMode::Static => {
                            let fan_handles = self
                                .set_static_fan_control(settings.static_speed)
                                .await
                                .context("Failed to set static fan control")?;

                            for handle in fan_handles {
                                commit_handles.push_front(handle);
                            }
                        }
                        lact_schema::FanControlMode::Curve => {
                            if settings.curve.0.is_empty() {
                                return Err(anyhow!("Cannot use empty fan curve"));
                            }

                            if let Some(commit_handle) = self
                                .start_curve_fan_control(settings.curve.clone(), settings.clone())
                                .await
                                .context("Failed to set curve fan control")?
                            {
                                commit_handles.push_front(commit_handle);
                            }
                        }
                    }
                } else {
                    return Err(anyhow!(
                        "Trying to enable fan control with no settings provided"
                    ));
                }
            } else {
                let pmfw = &config.pmfw_options;
                if let Some(acoustic_limit) = pmfw.acoustic_limit {
                    if self
                        .handle
                        .get_fan_acoustic_limit()
                        .context("Could not get acoustic limit")?
                        .current
                        != acoustic_limit
                    {
                        let commit_handle = self
                            .handle
                            .set_fan_acoustic_limit(acoustic_limit)
                            .context("Could not set acoustic limit")?;
                        commit_handles.push_front(commit_handle);
                    }
                }
                if let Some(acoustic_target) = pmfw.acoustic_target {
                    if self
                        .handle
                        .get_fan_acoustic_target()
                        .context("Could not get acoustic target")?
                        .current
                        != acoustic_target
                    {
                        let commit_handle = self
                            .handle
                            .set_fan_acoustic_target(acoustic_target)
                            .context("Could not set acoustic target")?;
                        commit_handles.push_front(commit_handle);
                    }
                }
                if let Some(target_temperature) = pmfw.target_temperature {
                    if self
                        .handle
                        .get_fan_target_temperature()
                        .context("Could not get target temperature")?
                        .current
                        != target_temperature
                    {
                        let commit_handle = self
                            .handle
                            .set_fan_target_temperature(target_temperature)
                            .context("Could not set target temperature")?;
                        commit_handles.push_front(commit_handle);
                    }
                }
                if let Some(minimum_pwm) = pmfw.minimum_pwm {
                    if self
                        .handle
                        .get_fan_minimum_pwm()
                        .context("Could not get minimum pwm")?
                        .current
                        != minimum_pwm
                    {
                        let commit_handle = self
                            .handle
                            .set_fan_minimum_pwm(minimum_pwm)
                            .context("Could not set minimum pwm")?;
                        commit_handles.push_front(commit_handle);
                    }
                }
            }

            // Unlike the other PMFW options, zero rpm should be applied with a custom curve as well (but not in static mode)
            if !(config.fan_control_enabled
                && config
                    .fan_control_settings
                    .as_ref()
                    .is_some_and(|fan| fan.mode == FanControlMode::Static))
            {
                if let Some(zero_rpm) = config.pmfw_options.zero_rpm {
                    match self.handle.get_fan_zero_rpm_enable() {
                        Ok(current_zero_rpm) => {
                            if current_zero_rpm != zero_rpm {
                                let commit_handle = self
                                    .handle
                                    .set_fan_zero_rpm_enable(zero_rpm)
                                    .context("Could not set zero RPM mode")?;
                                commit_handles.push_front(commit_handle);
                            }
                        }
                        Err(err) => {
                            error!("zero RPM is present in the config, but not available on the GPU: {err}");
                        }
                    }
                }

                if let Some(zero_rpm_threshold) = config.pmfw_options.zero_rpm_threshold {
                    match self.handle.get_fan_zero_rpm_stop_temperature() {
                        Ok(current_threshold) => {
                            if current_threshold.current != zero_rpm_threshold {
                                let commit_handle = self
                                    .handle
                                    .set_fan_zero_rpm_stop_temperature(zero_rpm_threshold)
                                    .context("Could not set zero RPM temperature")?;
                                commit_handles.push_front(commit_handle);
                            }
                        }
                        Err(err) => {
                            error!("zero RPM threshold is present in the config, but not available on the GPU: {err}");
                        }
                    }
                }
            }

            if let Some(configured_cap) = config.power_cap {
                let hw_mon = self.first_hw_mon()?;

                hw_mon
                    .set_power_cap(configured_cap)
                    .with_context(|| format!("Failed to set power cap: {configured_cap}"))?;
            } else if let Ok(hw_mon) = self.first_hw_mon() {
                if let Ok(default_cap) = hw_mon.get_power_cap_default() {
                    if Ok(default_cap) != hw_mon.get_power_cap() {
                        hw_mon.set_power_cap(default_cap).with_context(|| {
                            format!("Failed to set power cap to default cap: {default_cap}")
                        })?;
                    }
                }
            }

            for handle in commit_handles {
                handle.commit()?;
            }

            if let Some(performance_level) = deferred_performance_level {
                self.handle
                    .set_power_force_performance_level(performance_level)
                    .context("Failed to set power performance level")?;
            }

            for (kind, states) in &config.power_states {
                if config.performance_level != Some(PerformanceLevel::Manual) {
                    return Err(anyhow!(
                        "Performance level has to be set to `manual` to configure power states"
                    ));
                }

                self.handle
                    .set_enabled_power_levels(*kind, states)
                    .with_context(|| format!("Could not set {kind:?} power states"))?;
            }

            Ok(())
        })
    }

    fn reset_clocks(&self) -> anyhow::Result<()> {
        if self.handle.get_clocks_table().is_err() {
            return Ok(());
        }

        self.handle.reset_clocks_table()?;

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
        let mut last_total_time_map = self.last_drm_util.borrow_mut();
        fdinfo::read_process_list(
            &self.common,
            DRM_VRAM_KEYS,
            DRM_ENGINES,
            &mut last_total_time_map,
        )
    }
}

#[cfg(not(test))]
fn get_drm_handle(
    common: &CommonControllerInfo,
    libdrm_amdgpu: &LibDrmAmdgpu,
) -> anyhow::Result<DrmHandle> {
    use std::os::unix::io::IntoRawFd;

    let path = common.get_drm_render()?;
    let drm_file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&path)
        .with_context(|| format!("Could not open drm file at {}", path.display()))?;
    let (handle, _, _) = libdrm_amdgpu
        .init_device_handle(drm_file.into_raw_fd())
        .map_err(|err| anyhow!("Could not open drm handle, error code {err}"))?;
    Ok(handle)
}

fn apply_clocks_config_to_table(
    config: &ClocksConfiguration,
    table: &mut ClocksTableGen,
) -> anyhow::Result<()> {
    if let ClocksTableGen::Rdna(ref mut table) = table {
        // Avoid writing settings to the clocks table except the user-specified ones
        // There is an issue on some GPU models where the default values are actually outside of the allowed range
        // See https://github.com/sibradzic/amdgpu-clocks/issues/32#issuecomment-829953519 (part 2) for an example
        table.clear();

        // Normalize the VDDC curve - make sure all of the values are within the allowed range
        table.normalize_vddc_curve();

        match config.voltage_offset {
            Some(offset) => table.set_voltage_offset(offset)?,
            None => table.voltage_offset = None,
        }

        if let Some(offset) = config.gpu_clock_offsets.get(&0) {
            table.sclk_offset = Some(*offset);
        }
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

    if !config.gpu_vf_curve.is_empty() {
        // Validate first
        for (i, point) in &config.gpu_vf_curve {
            if let Some(clockspeed) = point.clockspeed {
                let range = match table {
                    ClocksTableGen::Gcn(table) => table.od_range.sclk,
                    ClocksTableGen::Rdna(table) => table
                        .od_range
                        .curve_sclk_points
                        .get(*i as usize)
                        .copied()
                        .unwrap_or_default(),
                };
                let (min, max) = range.into_full().context("SCLK range not present")?;

                if !(min..=max).contains(&clockspeed) {
                    bail!("Clockspeed {clockspeed} is not in the allowed range {min}-{max}");
                }
            }

            if let Some(voltage) = point.voltage {
                let range = match table {
                    ClocksTableGen::Gcn(table) => table.od_range.vddc,
                    ClocksTableGen::Rdna(table) => table
                        .od_range
                        .curve_voltage_points
                        .get(*i as usize)
                        .copied(),
                };
                let (min, max) = range
                    .unwrap_or_default()
                    .into_full()
                    .context("Voltage range not present")?;

                if !(min..=max).contains(&voltage) {
                    bail!("Voltage {voltage} is not in the allowed range {min}-{max}");
                }
            }
        }

        let points = match table {
            ClocksTableGen::Gcn(table) => &mut table.sclk_levels,
            ClocksTableGen::Rdna(table) => &mut table.vddc_curve,
        };
        for (i, point) in &config.gpu_vf_curve {
            let level = points
                .get_mut(*i as usize)
                .with_context(|| format!("GPU does not have a p-state {i}"))?;

            if let Some(clockspeed) = point.clockspeed {
                level.clockspeed = clockspeed;
            }

            if let Some(voltage) = point.voltage {
                level.voltage = voltage;
            }
        }

        // Apply RDNA1 OD_SCLK options in addition to the vddc curve
        if let ClocksTableGen::Rdna(table) = table {
            if let Some(min_clock) = config.gpu_vf_curve.get(&0) {
                table.current_sclk_range.min = min_clock.clockspeed;
            }

            #[allow(clippy::cast_possible_truncation)]
            if let Some(max_clock) = config.gpu_vf_curve.get(&(table.vddc_curve.len() as u8 - 1)) {
                table.current_sclk_range.max = max_clock.clockspeed;
            }
        }
    }

    if !config.mem_vf_curve.is_empty() {
        match table {
            ClocksTableGen::Gcn(table) => {
                // Validate first
                for (_, point) in &config.mem_vf_curve {
                    if let Some(clockspeed) = point.clockspeed {
                        let (min, max) = table
                            .od_range
                            .mclk
                            .unwrap_or_default()
                            .into_full()
                            .context("MCLK range not present")?;

                        if !(min..=max).contains(&clockspeed) {
                            bail!(
                                "Memory clockspeed {clockspeed} is not in the allowed range {min}-{max}"
                            );
                        }
                    }

                    if let Some(voltage) = point.voltage {
                        let (min, max) = table
                            .od_range
                            .vddc
                            .unwrap_or_default()
                            .into_full()
                            .context("Voltage range not present")?;

                        if !(min..=max).contains(&voltage) {
                            bail!("Voltage {voltage} is not in the allowed range {min}-{max}");
                        }
                    }
                }

                for (i, point) in &config.mem_vf_curve {
                    let level = table
                        .mclk_levels
                        .get_mut(*i as usize)
                        .with_context(|| format!("GPU does not have a p-state {i}"))?;

                    if let Some(clockspeed) = point.clockspeed {
                        level.clockspeed = clockspeed;
                    }

                    if let Some(voltage) = point.voltage {
                        level.voltage = voltage;
                    }
                }
            }
            ClocksTableGen::Rdna(_) => bail!("VRAM VF curve not supported"),
        }
    }

    Ok(())
}

fn read_cache_info_from_kfd(node_path: &Path) -> anyhow::Result<CacheInfo> {
    let mut caches: BTreeMap<AmdCacheInstance, u16> = BTreeMap::new();

    let caches_dir = fs::read_dir(node_path.join("caches"))?;
    for entry in caches_dir {
        let entry = entry?;

        let contents = fs::read_to_string(entry.path().join("properties"))
            .context("Could not read cache properties")?;
        let cache_info = contents
            .lines()
            .filter_map(|line| line.split_once(' '))
            .collect::<HashMap<_, _>>();

        if let (Some(cache_type), Some(level), Some(size), Some(sibling_map)) = (
            cache_info.get("type"),
            cache_info.get("level"),
            cache_info.get("size"),
            cache_info.get("sibling_map"),
        ) {
            let level: u8 = level.parse().context("Invalid cache size")?;
            let size: u32 = size.parse().context("Invalid cache size")?;
            let cache_type: u32 = cache_type.parse().context("Invalid cache size")?;
            #[allow(clippy::cast_possible_truncation)]
            let cu_count = sibling_map
                .split(',')
                .take_while(|item| *item == "1")
                .count() as u16;

            let types = [
                (HSA_CACHE_TYPE_DATA, CacheType::Data),
                (HSA_CACHE_TYPE_INSTRUCTION, CacheType::Instruction),
                (HSA_CACHE_TYPE_CPU, CacheType::Cpu),
            ]
            .into_iter()
            .filter(|(mask, _)| cache_type & mask > 0)
            .map(|(_, cache_type)| cache_type)
            .collect();

            let instance = AmdCacheInstance {
                level,
                types,
                size: size * 1024,
                cu_count,
            };
            *caches.entry(instance).or_default() += 1;
        }
    }
    let mut items = caches.into_iter().collect::<Vec<_>>();

    items.sort_by_key(|(instance, _)| {
        (
            instance.level,
            instance.cu_count,
            instance.size,
            instance.types.clone(),
        )
    });

    Ok(CacheInfo::Amd(items))
}
