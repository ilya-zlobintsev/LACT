use super::{
    gpu_controller::{fan_control::FanCurve, GpuController},
    system::{self, detect_initramfs_type, PP_FEATURE_MASK_PATH},
};
use crate::{
    config::{self, default_fan_static_speed, Config, FanControlSettings, Profile},
    server::gpu_controller::{AmdGpuController, NvidiaGpuController},
};
use amdgpu_sysfs::{
    gpu_handle::{power_profile_mode::PowerProfileModesTable, PerformanceLevel, PowerLevelKind},
    sysfs::SysFS,
};
use anyhow::{anyhow, bail, Context};
use lact_schema::{
    default_fan_curve,
    request::{ConfirmCommand, ProfileBase, SetClocksCommand},
    ClocksInfo, DeviceInfo, DeviceListEntry, DeviceStats, FanControlMode, FanOptions, PmfwOptions,
    PowerStates, ProfilesInfo,
};
use libflate::gzip;
use nix::libc;
use nvml_wrapper::{error::NvmlError, Nvml};
use os_release::OS_RELEASE;
use pciid_parser::Database;
use serde_json::json;
use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap},
    env,
    fs::{self, File, Permissions},
    io::{BufWriter, Cursor, Write},
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::{Path, PathBuf},
    rc::Rc,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::{process::Command, sync::oneshot, time::sleep};
use tracing::{debug, error, info, trace, warn};

const CONTROLLERS_LOAD_RETRY_ATTEMPTS: u8 = 5;
const CONTROLLERS_LOAD_RETRY_INTERVAL: u64 = 3;

const SNAPSHOT_GLOBAL_FILES: &[&str] = &[
    PP_FEATURE_MASK_PATH,
    "/etc/lact/config.yaml",
    "/proc/version",
];
const SNAPSHOT_DEVICE_FILES: &[&str] = &[
    "uevent",
    "vendor",
    "pp_cur_state",
    "pp_dpm_mclk",
    "pp_dpm_pcie",
    "pp_dpm_sclk",
    "pp_dpm_socclk",
    "pp_features",
    "pp_force_state",
    "pp_mclk_od",
    "pp_num_states",
    "pp_od_clk_voltage",
    "pp_power_profile_mode",
    "pp_sclk_od",
    "pp_table",
    "vbios_version",
    "gpu_busy_percent",
    "current_link_speed",
    "current_link_width",
];
/// Prefixes for entries that will be recursively included in the debug snapshot
const SNAPSHOT_DEVICE_RECURSIVE_PATHS_PREFIXES: &[&str] = &["tile"];
const SNAPSHOT_FAN_CTRL_FILES: &[&str] = &[
    "fan_curve",
    "acoustic_limit_rpm_threshold",
    "acoustic_target_rpm_threshold",
    "fan_minimum_pwm",
    "fan_target_temperature",
    "fan_zero_rpm_enable",
    "fan_zero_rpm_stop_temperature",
];
const SNAPSHOT_HWMON_FILE_PREFIXES: &[&str] =
    &["fan", "pwm", "power", "temp", "freq", "in", "name"];

#[derive(Clone)]
pub struct Handler {
    pub config: Rc<RefCell<Config>>,
    pub gpu_controllers: Rc<BTreeMap<String, Box<dyn GpuController>>>,
    confirm_config_tx: Rc<RefCell<Option<oneshot::Sender<ConfirmCommand>>>>,
    pub config_last_saved: Arc<Mutex<Instant>>,
}

impl<'a> Handler {
    pub async fn new(config: Config) -> anyhow::Result<Self> {
        let base_path = match env::var("_LACT_DRM_SYSFS_PATH") {
            Ok(custom_path) => PathBuf::from(custom_path),
            Err(_) => PathBuf::from("/sys/class/drm"),
        };
        Self::with_base_path(&base_path, config, false).await
    }

    pub(crate) async fn with_base_path(
        base_path: &Path,
        config: Config,
        sysfs_only: bool,
    ) -> anyhow::Result<Self> {
        let mut controllers = BTreeMap::new();

        // Sometimes LACT starts too early in the boot process, before the sysfs is initialized.
        // For such scenarios there is a retry logic when no GPUs were found,
        // or if some of the PCI devices don't have a drm entry yet.
        for i in 1..=CONTROLLERS_LOAD_RETRY_ATTEMPTS {
            controllers = load_controllers(base_path, sysfs_only)?;

            let mut should_retry = false;
            if let Ok(devices) = fs::read_dir("/sys/bus/pci/devices") {
                for device in devices.flatten() {
                    if let Ok(uevent) = fs::read_to_string(device.path().join("uevent")) {
                        let uevent = uevent.replace('\0', "");
                        if uevent.contains("amdgpu") || uevent.contains("radeon") {
                            let slot_name = device
                                .file_name()
                                .into_string()
                                .expect("pci file name should be valid unicode");

                            if controllers.values().any(|controller| {
                                controller.get_pci_slot_name().as_ref() == Some(&slot_name)
                            }) {
                                debug!("found intialized drm entry for device {:?}", device.path());
                            } else {
                                warn!("could not find drm entry for device {:?}", device.path());
                                should_retry = true;
                            }
                        }
                    }
                }
            }

            if controllers.is_empty() {
                warn!("no GPUs were found");
                should_retry = true;
            }

            if should_retry {
                info!("retrying in {CONTROLLERS_LOAD_RETRY_INTERVAL}s (attempt {i}/{CONTROLLERS_LOAD_RETRY_ATTEMPTS})");
                sleep(Duration::from_secs(CONTROLLERS_LOAD_RETRY_INTERVAL)).await;
            } else {
                break;
            }
        }
        info!("initialized {} GPUs", controllers.len());

        let handler = Self {
            gpu_controllers: Rc::new(controllers),
            config: Rc::new(RefCell::new(config)),
            confirm_config_tx: Rc::new(RefCell::new(None)),
            config_last_saved: Arc::new(Mutex::new(Instant::now())),
        };
        if let Err(err) = handler.apply_current_config().await {
            error!("could not apply config: {err:#}");
        }

        // Eagerly release memory
        // `load_controllers` allocates and deallocates the entire PCI ID database,
        // this tells the os to release it right away, lowering measured memory usage (the actual usage is low regardless as it was already deallocated)
        #[cfg(target_env = "gnu")]
        unsafe {
            libc::malloc_trim(0);
        }

        Ok(handler)
    }

    pub async fn apply_current_config(&self) -> anyhow::Result<()> {
        let config = self.config.borrow().clone(); // Clone to avoid locking the RwLock on an await point

        let gpus = config.gpus()?;
        for (id, gpu_config) in gpus {
            if let Some(controller) = self.gpu_controllers.get(id) {
                if let Err(err) = controller.apply_config(gpu_config).await {
                    error!("could not apply existing config for gpu {id}: {err}");
                }
            } else {
                info!("could not find GPU with id {id} defined in configuration");
            }
        }

        Ok(())
    }

    async fn edit_gpu_config<F: FnOnce(&mut config::Gpu)>(
        &self,
        id: String,
        f: F,
    ) -> anyhow::Result<u64> {
        if self
            .confirm_config_tx
            .try_borrow_mut()
            .map_err(|err| anyhow!("{err}"))?
            .is_some()
        {
            return Err(anyhow!(
                "There is an unconfirmed configuration change pending"
            ));
        }

        let (gpu_config, apply_timer) = {
            let config = self.config.try_borrow().map_err(|err| anyhow!("{err}"))?;
            let apply_timer = config.apply_settings_timer;
            let gpu_config = config.gpus()?.get(&id).cloned().unwrap_or_default();
            (gpu_config, apply_timer)
        };

        let mut new_config = gpu_config.clone();
        f(&mut new_config);

        let controller = self.controller_by_id(&id)?;

        match controller.apply_config(&new_config).await {
            Ok(()) => {
                self.wait_config_confirm(id, gpu_config, new_config, apply_timer)?;
                Ok(apply_timer)
            }
            Err(apply_err) => {
                error!("could not apply settings: {apply_err:?}");
                match controller.apply_config(&gpu_config).await {
                    Ok(()) => Err(apply_err.context("Could not apply settings")),
                    Err(err) => Err(apply_err.context(err.context(
                        "Could not apply settings, and could not reset to default settings",
                    ))),
                }
            }
        }
    }

    /// Should be called after applying new config without writing it
    fn wait_config_confirm(
        &self,
        id: String,
        previous_config: config::Gpu,
        new_config: config::Gpu,
        apply_timer: u64,
    ) -> anyhow::Result<()> {
        let (tx, rx) = oneshot::channel();
        *self
            .confirm_config_tx
            .try_borrow_mut()
            .map_err(|err| anyhow!("{err}"))? = Some(tx);

        let handler = self.clone();

        tokio::task::spawn_local(async move {
            let controller = handler
                .controller_by_id(&id)
                .expect("GPU controller disappeared");

            tokio::select! {
                () = tokio::time::sleep(Duration::from_secs(apply_timer)) => {
                    info!("no confirmation received, reverting settings");

                    if let Err(err) = controller.apply_config(&previous_config).await {
                        error!("could not revert settings: {err:#}");
                    }
                }
                result = rx => {
                    match result {
                        Ok(ConfirmCommand::Confirm) => {
                            info!("saving updated config");
                            let mut config_guard = handler.config.borrow_mut();
                            match config_guard.gpus_mut() {
                                Ok(gpus) => {
                                    gpus.insert(id, new_config);
                                }
                                Err(err) => error!("{err:#}"),
                            }

                            if let Err(err) = config_guard.save(&handler.config_last_saved) {
                                error!("{err:#}");
                            }
                        }
                        Ok(ConfirmCommand::Revert) | Err(_) => {
                            if let Err(err) = controller.apply_config(&previous_config).await {
                                error!("could not revert settings: {err:#}");
                            }
                        }
                    }
                }
            }

            match handler.confirm_config_tx.try_borrow_mut() {
                Ok(mut guard) => *guard = None,
                Err(err) => error!("{err}"),
            }
        });

        Ok(())
    }

    fn controller_by_id(&self, id: &str) -> anyhow::Result<&dyn GpuController> {
        Ok(self
            .gpu_controllers
            .get(id)
            .context("No controller with such id")?
            .as_ref())
    }

    pub fn list_devices(&'a self) -> Vec<DeviceListEntry> {
        self.gpu_controllers
            .iter()
            .map(|(id, controller)| {
                let name = controller
                    .get_pci_info()
                    .and_then(|pci_info| pci_info.device_pci_info.model.clone());
                DeviceListEntry {
                    id: id.to_owned(),
                    name,
                }
            })
            .collect()
    }

    pub fn get_device_info(&'a self, id: &str) -> anyhow::Result<DeviceInfo> {
        Ok(self.controller_by_id(id)?.get_info())
    }

    pub fn get_gpu_stats(&'a self, id: &str) -> anyhow::Result<DeviceStats> {
        let config = self
            .config
            .try_borrow()
            .map_err(|err| anyhow!("Could not read config: {err:?}"))?;
        let gpu_config = config.gpus()?.get(id);
        Ok(self.controller_by_id(id)?.get_stats(gpu_config))
    }

    pub fn get_clocks_info(&'a self, id: &str) -> anyhow::Result<ClocksInfo> {
        self.controller_by_id(id)?.get_clocks_info()
    }

    pub async fn set_fan_control(&'a self, opts: FanOptions<'_>) -> anyhow::Result<u64> {
        let settings = {
            let mut config_guard = self
                .config
                .try_borrow_mut()
                .map_err(|err| anyhow!("{err}"))?;
            let gpu_config = config_guard
                .gpus_mut()?
                .entry(opts.id.to_owned())
                .or_default();

            match opts.mode {
                Some(mode) => match mode {
                    FanControlMode::Static => {
                        if matches!(opts.static_speed, Some(speed) if !(0.0..=1.0).contains(&speed))
                        {
                            return Err(anyhow!("static speed value out of range"));
                        }

                        if let Some(mut existing_settings) = gpu_config.fan_control_settings.clone()
                        {
                            existing_settings.mode = mode;
                            if let Some(static_speed) = opts.static_speed {
                                existing_settings.static_speed = static_speed;
                            }
                            Some(existing_settings)
                        } else {
                            Some(FanControlSettings {
                                mode,
                                static_speed: opts
                                    .static_speed
                                    .unwrap_or_else(default_fan_static_speed),
                                ..Default::default()
                            })
                        }
                    }
                    FanControlMode::Curve => {
                        if let Some(mut existing_settings) = gpu_config.fan_control_settings.clone()
                        {
                            existing_settings.mode = mode;
                            if let Some(change_threshold) = opts.change_threshold {
                                existing_settings.change_threshold = Some(change_threshold);
                            }
                            if let Some(spindown_delay) = opts.spindown_delay_ms {
                                existing_settings.spindown_delay_ms = Some(spindown_delay);
                            }

                            if let Some(raw_curve) = opts.curve {
                                let curve = FanCurve(raw_curve);
                                curve.validate()?;
                                existing_settings.curve = curve;
                            }
                            Some(existing_settings)
                        } else {
                            let curve = FanCurve(opts.curve.unwrap_or_else(default_fan_curve));
                            curve.validate()?;
                            Some(FanControlSettings {
                                mode,
                                curve,
                                change_threshold: opts.change_threshold,
                                spindown_delay_ms: opts.spindown_delay_ms,
                                ..Default::default()
                            })
                        }
                    }
                },
                None => None,
            }
        };

        self.edit_gpu_config(opts.id.to_owned(), |config| {
            config.fan_control_enabled = opts.enabled;
            if let Some(settings) = settings {
                config.fan_control_settings = Some(settings);
            }
            config.pmfw_options = opts.pmfw;
        })
        .await
        .context("Failed to edit GPU config")
    }

    pub async fn reset_pmfw(&self, id: &str) -> anyhow::Result<u64> {
        info!("Resetting PMFW settings");
        self.controller_by_id(id)?.reset_pmfw_settings();

        self.edit_gpu_config(id.to_owned(), |config| {
            config.pmfw_options = PmfwOptions::default();
        })
        .await
        .context("Failed to edit GPU config and reset pmfw")
    }

    pub async fn set_power_cap(&'a self, id: &str, maybe_cap: Option<f64>) -> anyhow::Result<u64> {
        self.edit_gpu_config(id.to_owned(), |gpu_config| {
            gpu_config.power_cap = maybe_cap;
        })
        .await
        .context("Failed to edit GPU config and set power cap")
    }

    pub fn get_power_states(&self, id: &str) -> anyhow::Result<PowerStates> {
        let config = self
            .config
            .try_borrow()
            .map_err(|err| anyhow!("Could not read config: {err:?}"))?;
        let gpu_config = config.gpus()?.get(id);

        let states = self.controller_by_id(id)?.get_power_states(gpu_config);
        Ok(states)
    }

    pub async fn set_performance_level(
        &self,
        id: &str,
        level: PerformanceLevel,
    ) -> anyhow::Result<u64> {
        self.edit_gpu_config(id.to_owned(), |gpu_config| {
            gpu_config.performance_level = Some(level);

            if level != PerformanceLevel::Manual {
                gpu_config.power_states.clear();
            }
        })
        .await
        .context("Failed to edit GPU config and set performance level")
    }

    pub async fn set_clocks_value(
        &self,
        id: &str,
        command: SetClocksCommand,
    ) -> anyhow::Result<u64> {
        if let SetClocksCommand::Reset = command {
            self.controller_by_id(id)?.cleanup_clocks()?;
        }

        self.edit_gpu_config(id.to_owned(), |gpu_config| {
            gpu_config.apply_clocks_command(&command);
        })
        .await
        .context("Failed to edit GPU config and set clocks value")
    }

    pub async fn batch_set_clocks_value(
        &self,
        id: &str,
        commands: Vec<SetClocksCommand>,
    ) -> anyhow::Result<u64> {
        self.edit_gpu_config(id.to_owned(), |gpu_config| {
            for command in commands {
                gpu_config.apply_clocks_command(&command);
            }
        })
        .await
        .context("Failed to edit GPU config and batch set clocks")
    }

    pub fn get_power_profile_modes(&self, id: &str) -> anyhow::Result<PowerProfileModesTable> {
        let modes_table = self.controller_by_id(id)?.get_power_profile_modes()?;
        Ok(modes_table)
    }

    pub async fn set_power_profile_mode(
        &self,
        id: &str,
        index: Option<u16>,
        custom_heuristics: Vec<Vec<Option<i32>>>,
    ) -> anyhow::Result<u64> {
        self.edit_gpu_config(id.to_owned(), |gpu_config| {
            gpu_config.power_profile_mode_index = index;
            gpu_config.custom_power_profile_mode_hueristics = custom_heuristics;
        })
        .await
        .context("Failed to edit GPU config and set power profile mode")
    }

    pub async fn set_enabled_power_states(
        &self,
        id: &str,
        kind: PowerLevelKind,
        enabled_states: Vec<u8>,
    ) -> anyhow::Result<u64> {
        self.edit_gpu_config(id.to_owned(), |gpu| {
            gpu.power_states.insert(kind, enabled_states);
        })
        .await
        .context("Failed to edit GPU config and set enabled power states")
    }

    pub fn vbios_dump(&self, id: &str) -> anyhow::Result<Vec<u8>> {
        self.controller_by_id(id)?.vbios_dump()
    }

    pub async fn generate_snapshot(&self) -> anyhow::Result<String> {
        let datetime = chrono::Local::now().format("%Y%m%d-%H%M%S");
        let out_path = format!("/tmp/LACT-sysfs-snapshot-{datetime}.tar.gz");

        let out_file = File::create(&out_path)
            .with_context(|| "Could not create output file at {out_path}")?;
        let out_writer = gzip::Encoder::new(BufWriter::new(out_file))
            .context("Could not create GZIP encoder")?;

        let mut archive = tar::Builder::new(out_writer);

        for path in SNAPSHOT_GLOBAL_FILES {
            let path = Path::new(path);
            add_path_to_archive(&mut archive, path)?;
        }

        for controller in self.gpu_controllers.values() {
            let controller_path = controller.get_path();

            for device_file in SNAPSHOT_DEVICE_FILES {
                let full_path = controller_path.join(device_file);
                add_path_to_archive(&mut archive, &full_path)?;
            }

            let device_files = fs::read_dir(controller.get_path())
                .context("Could not read device dir")?
                .flatten();

            for device_entry in device_files {
                if let Some(entry_name) = device_entry.file_name().to_str() {
                    if SNAPSHOT_DEVICE_RECURSIVE_PATHS_PREFIXES
                        .iter()
                        .any(|prefix| entry_name.starts_with(prefix))
                    {
                        add_path_recursively(
                            &mut archive,
                            &device_entry.path(),
                            controller.get_path(),
                        )?;
                    }
                }
            }

            let fan_ctrl_path = controller_path.join("gpu_od").join("fan_ctrl");
            for fan_ctrl_file in SNAPSHOT_FAN_CTRL_FILES {
                let full_path = fan_ctrl_path.join(fan_ctrl_file);
                add_path_to_archive(&mut archive, &full_path)?;
            }

            for hw_mon in controller.hw_monitors() {
                let hw_mon_path = hw_mon.get_path();
                let hw_mon_entries =
                    fs::read_dir(hw_mon_path).context("Could not read HwMon dir")?;

                'entries: for entry in hw_mon_entries.flatten() {
                    if !entry.metadata().is_ok_and(|metadata| metadata.is_file()) {
                        continue;
                    }

                    if let Some(name) = entry.file_name().to_str() {
                        for prefix in SNAPSHOT_HWMON_FILE_PREFIXES {
                            if name.starts_with(prefix) {
                                add_path_to_archive(&mut archive, &entry.path())?;
                                continue 'entries;
                            }
                        }
                    }
                }
            }
        }

        let service_journal_output = Command::new("journalctl")
            .args(["-u", "lactd", "-b"])
            .output()
            .await;

        match service_journal_output {
            Ok(output) => {
                if !output.status.success() {
                    warn!("service log output has status code {}", output.status);
                }
                let mut header = tar::Header::new_gnu();
                header.set_size(output.stdout.len().try_into().unwrap());
                header.set_mode(0o755);
                header.set_cksum();

                archive
                    .append_data(&mut header, "lactd.log", Cursor::new(output.stdout))
                    .context("Could not write data to archive")?;
            }
            Err(err) => warn!("could not read service log: {err}"),
        }

        let system_info = system::info()
            .await
            .ok()
            .map(|info| serde_json::to_value(info).unwrap());
        let initramfs_type = match OS_RELEASE.as_ref() {
            Ok(os_release) => detect_initramfs_type(os_release)
                .await
                .map(|initramfs_type| serde_json::to_value(initramfs_type).unwrap()),
            Err(err) => Some(err.to_string().into()),
        };

        let info = json!({
            "system_info": system_info,
            "initramfs_type": initramfs_type,
            "devices": self.generate_snapshot_device_info(),
        });
        let info_data = serde_json::to_vec_pretty(&info).unwrap();

        let mut info_header = tar::Header::new_gnu();
        info_header.set_size(info_data.len().try_into().unwrap());
        info_header.set_mode(0o755);
        info_header.set_cksum();

        archive.append_data(&mut info_header, "info.json", Cursor::new(info_data))?;

        let mut writer = archive.into_inner().context("Could not finish archive")?;
        writer.flush().context("Could not flush output file")?;

        writer
            .finish()
            .into_result()
            .context("Could not finish GZIP archive")?
            .into_inner()?
            .set_permissions(Permissions::from_mode(0o775))
            .context("Could not set permissions on output file")?;

        Ok(out_path)
    }

    pub(crate) fn generate_snapshot_device_info(&self) -> BTreeMap<String, serde_json::Value> {
        self.gpu_controllers
            .iter()
            .map(|(id, controller)| {
                let config = self.config.try_borrow();
                let gpu_config = config
                    .as_ref()
                    .ok()
                    .and_then(|config| config.gpus().ok()?.get(id));

                let data = json!({
                    "pci_info": controller.get_pci_info(),
                    "info": controller.get_info(),
                    "stats": controller.get_stats(gpu_config),
                    "clocks_info": controller.get_clocks_info().ok(),
                    "power_profile_modes": controller.get_power_profile_modes().ok(),
                });
                (id.clone(), data)
            })
            .collect()
    }

    pub fn list_profiles(&self) -> ProfilesInfo {
        let config = self.config.borrow();
        ProfilesInfo {
            profiles: config.profiles.keys().cloned().collect(),
            current_profile: config.current_profile.clone(),
        }
    }

    pub async fn set_profile(&self, name: Option<String>) -> anyhow::Result<()> {
        if let Some(name) = &name {
            self.config.borrow().profile(name)?;
        }

        self.cleanup().await;
        self.config.borrow_mut().current_profile = name;

        self.apply_current_config().await?;
        self.config.borrow_mut().save(&self.config_last_saved)?;

        Ok(())
    }

    pub fn create_profile(&self, name: String, base: ProfileBase) -> anyhow::Result<()> {
        let mut config = self.config.borrow_mut();
        if config.profiles.contains_key(&name) {
            bail!("Profile {name} already exists");
        }

        let profile = match base {
            ProfileBase::Empty => Profile::default(),
            ProfileBase::Default => config.default_profile(),
            ProfileBase::Profile(name) => config.profile(&name)?.clone(),
        };
        config.profiles.insert(name, profile);
        config.save(&self.config_last_saved)?;
        Ok(())
    }

    pub async fn delete_profile(&self, name: String) -> anyhow::Result<()> {
        if self.config.borrow().current_profile.as_ref() == Some(&name) {
            self.set_profile(None).await?;
        }
        self.config.borrow_mut().profiles.shift_remove(&name);
        self.config.borrow().save(&self.config_last_saved)?;
        Ok(())
    }

    pub fn confirm_pending_config(&self, command: ConfirmCommand) -> anyhow::Result<()> {
        if let Some(tx) = self
            .confirm_config_tx
            .try_borrow_mut()
            .map_err(|err| anyhow!("{err}"))?
            .take()
        {
            tx.send(command)
                .map_err(|_| anyhow!("Could not confirm config"))
        } else {
            Err(anyhow!("No pending config changes"))
        }
    }

    pub async fn reset_config(&self) {
        self.cleanup().await;

        let mut config = self.config.borrow_mut();
        config.clear();

        if let Err(err) = config.save(&self.config_last_saved) {
            error!("could not save config: {err:#}");
        }
    }

    pub async fn cleanup(&self) {
        let disable_clocks_cleanup = self
            .config
            .try_borrow()
            .map(|config| config.daemon.disable_clocks_cleanup)
            .unwrap_or(false);

        for (id, controller) in &*self.gpu_controllers {
            if !disable_clocks_cleanup {
                debug!("resetting clocks table");
                if let Err(err) = controller.cleanup_clocks() {
                    error!("could not reset the clocks table: {err}");
                }
            }

            controller.reset_pmfw_settings();

            if let Err(err) = controller.apply_config(&config::Gpu::default()).await {
                error!("Could not reset settings for controller {id}: {err:#}");
            }
        }
    }
}

/// `sysfs_only` disables initialization of any external data sources, such as libdrm and nvml
fn load_controllers(
    base_path: &Path,
    sysfs_only: bool,
) -> anyhow::Result<BTreeMap<String, Box<dyn GpuController>>> {
    let mut controllers = BTreeMap::new();

    let pci_db = Database::read().unwrap_or_else(|err| {
        warn!("could not read PCI ID database: {err}, device information will be limited");
        Database {
            vendors: HashMap::new(),
            classes: HashMap::new(),
        }
    });

    let nvml = if sysfs_only {
        None
    } else {
        match Nvml::init() {
            Ok(nvml) => {
                info!("NVML initialized");
                Some(Rc::new(nvml))
            }
            Err(err) => {
                info!("Nvidia support disabled, {err}");
                None
            }
        }
    };

    for entry in base_path
        .read_dir()
        .map_err(|error| anyhow!("Failed to read sysfs: {error}"))?
    {
        let entry = entry?;

        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| anyhow!("non-utf path"))?;
        if name.starts_with("card") && !name.contains('-') {
            trace!("trying gpu controller at {:?}", entry.path());
            let device_path = entry.path().join("device");
            match AmdGpuController::new_from_path(device_path, &pci_db, sysfs_only) {
                Ok(controller) => match controller.get_id() {
                    Ok(id) => {
                        let path = controller.get_path();

                        if let Some(nvml) = nvml.clone() {
                            if let Some(pci_slot_id) = controller.get_pci_slot_name() {
                                match nvml.device_by_pci_bus_id(pci_slot_id.as_str()) {
                                    Ok(_) => {
                                        let controller = NvidiaGpuController::new(
                                            nvml,
                                            pci_slot_id,
                                             controller.get_pci_info().expect(
                                                "Initialized NVML device without PCI info somehow",
                                            ).clone(),
                                             path.to_owned(),
                                        );
                                        match controller.get_id() {
                                            Ok(id) => {
                                                info!("initialized Nvidia GPU controller {id} for path {path:?}");
                                                controllers.insert(
                                                    id,
                                                    Box::new(controller) as Box<dyn GpuController>,
                                                );
                                                continue;
                                            }
                                            Err(err) => {
                                                error!("could not get Nvidia GPU id: {err}");
                                            }
                                        }
                                    }
                                    Err(NvmlError::NotFound) => {
                                        debug!("PCI slot {pci_slot_id} not found in NVML");
                                    }
                                    Err(err) => {
                                        error!(
                                            "could not initialize Nvidia GPU at {path:?}: {err}"
                                        );
                                    }
                                }
                            }
                        }

                        info!("initialized GPU controller {id} for path {path:?}");
                        controllers.insert(id, Box::new(controller) as Box<dyn GpuController>);
                    }
                    Err(err) => warn!("could not initialize controller: {err:#}"),
                },
                Err(error) => {
                    warn!(
                        "failed to initialize controller at {:?}, {error}",
                        entry.path()
                    );
                }
            }
        }
    }

    Ok(controllers)
}

fn add_path_recursively(
    archive: &mut tar::Builder<impl Write>,
    entry_path: &Path,
    controller_path: &Path,
) -> anyhow::Result<()> {
    if let Ok(entries) = fs::read_dir(entry_path) {
        for entry in entries.flatten() {
            match entry.metadata() {
                Ok(metadata) => {
                    // Skip symlinks
                    if metadata.is_symlink() {
                        continue;
                    }

                    let full_path = controller_path.join(entry.path());
                    if metadata.is_file() {
                        add_path_to_archive(archive, &full_path)?;
                    } else if metadata.is_dir() {
                        add_path_recursively(archive, &full_path, controller_path)?;
                    }
                }
                Err(err) => {
                    warn!(
                        "could not include file '{}' in snapshot: {err}",
                        entry.path().display()
                    );
                }
            }
        }
    }

    Ok(())
}

fn add_path_to_archive(
    archive: &mut tar::Builder<impl Write>,
    full_path: &Path,
) -> anyhow::Result<()> {
    let archive_path = full_path
        .strip_prefix("/")
        .context("Path should always start at root")?;

    if let Ok(metadata) = std::fs::metadata(full_path) {
        debug!("adding {full_path:?} to snapshot");
        match std::fs::read(full_path) {
            Ok(data) => {
                let mut header = tar::Header::new_gnu();
                header.set_size(data.len().try_into().unwrap());
                header.set_mode(metadata.mode());
                header.set_uid(metadata.uid().into());
                header.set_gid(metadata.gid().into());
                header.set_cksum();

                archive
                    .append_data(&mut header, archive_path, Cursor::new(data))
                    .context("Could not write data to archive")?;
            }
            Err(err) => {
                warn!("file {full_path:?} exists, but could not be added to snapshot: {err}");
            }
        }
    }
    Ok(())
}
