use super::gpu_controller::{fan_control::FanCurve, GpuController};
use crate::config::{self, Config, FanControlSettings};
use anyhow::{anyhow, Context};
use lact_schema::{
    request::SetClocksCommand, ClocksInfo, DeviceInfo, DeviceListEntry, DeviceStats, FanCurveMap,
    PerformanceLevel,
};
use std::{
    collections::HashMap,
    env,
    path::PathBuf,
    sync::{Arc, RwLock},
};
use tracing::{debug, error, info, trace, warn};

#[derive(Clone)]
pub struct Handler {
    pub config: Arc<RwLock<Config>>,
    pub gpu_controllers: Arc<HashMap<String, GpuController>>,
}

impl<'a> Handler {
    pub async fn new(config: Config) -> anyhow::Result<Self> {
        let mut controllers = HashMap::new();

        let base_path = match env::var("_LACT_DRM_SYSFS_PATH") {
            Ok(custom_path) => PathBuf::from(custom_path),
            Err(_) => PathBuf::from("/sys/class/drm"),
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
                match GpuController::new_from_path(device_path) {
                    Ok(controller) => match controller.get_id() {
                        Ok(id) => {
                            let path = controller.get_path();
                            debug!("initialized GPU controller {id} for path {path:?}",);
                            controllers.insert(id, controller);
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

        for (id, gpu_config) in &config.gpus {
            if let Some(controller) = controllers.get(id) {
                if let Err(err) = controller.apply_config(gpu_config).await {
                    error!("could not apply existing config for gpu {id}: {err}");
                }
            } else {
                info!("could not find GPU with id {id} defined in configuration");
            }
        }

        Ok(Self {
            gpu_controllers: Arc::new(controllers),
            config: Arc::new(RwLock::new(config)),
        })
    }

    async fn edit_gpu_config<F: FnOnce(&mut config::Gpu)>(
        &self,
        id: String,
        f: F,
    ) -> anyhow::Result<()> {
        let current_config = self
            .config
            .read()
            .map_err(|err| anyhow!("{err}"))?
            .gpus
            .get(&id)
            .cloned()
            .unwrap_or_default();

        let mut new_config = current_config.clone();
        f(&mut new_config);

        let controller = self.controller_by_id(&id)?;

        match controller.apply_config(&new_config).await {
            Ok(()) => {
                let mut config_guard = self.config.write().unwrap();
                config_guard.gpus.insert(id, new_config);
                config_guard.save()?;
                Ok(())
            }
            Err(apply_err) => {
                error!("Could not apply settings: {apply_err:#}");
                match controller.apply_config(&current_config).await {
                    Ok(()) => Err(apply_err.context("Could not apply settings")),
                    Err(err) => Err(anyhow!("Could not apply settings, and could not reset to default settings: {err:#}")),
                }
            }
        }
    }

    fn controller_by_id(&self, id: &str) -> anyhow::Result<&GpuController> {
        Ok(self
            .gpu_controllers
            .get(id)
            .as_ref()
            .context("No controller with such id")?)
    }

    pub fn list_devices(&'a self) -> Vec<DeviceListEntry<'a>> {
        self.gpu_controllers
            .iter()
            .map(|(id, controller)| {
                let name = controller
                    .pci_info
                    .as_ref()
                    .and_then(|pci_info| pci_info.device_pci_info.model.as_deref());
                DeviceListEntry { id, name }
            })
            .collect()
    }

    pub fn get_device_info(&'a self, id: &str) -> anyhow::Result<DeviceInfo<'a>> {
        Ok(self.controller_by_id(id)?.get_info())
    }

    pub fn get_gpu_stats(&'a self, id: &str) -> anyhow::Result<DeviceStats> {
        let config = self
            .config
            .read()
            .map_err(|err| anyhow!("Could not read config: {err:?}"))?;
        let gpu_config = config.gpus.get(id);
        self.controller_by_id(id)?.get_stats(gpu_config)
    }

    pub fn get_clocks_info(&'a self, id: &str) -> anyhow::Result<ClocksInfo> {
        self.controller_by_id(id)?.get_clocks_info()
    }

    pub async fn set_fan_control(
        &'a self,
        id: &str,
        enabled: bool,
        curve: Option<FanCurveMap>,
    ) -> anyhow::Result<()> {
        let settings = match curve {
            Some(raw_curve) => {
                let curve = FanCurve(raw_curve);
                curve.validate()?;

                let mut config_guard = self.config.write().map_err(|err| anyhow!("{err}"))?;
                let gpu_config = config_guard.gpus.entry(id.to_owned()).or_default();

                if let Some(mut existing_settings) = gpu_config.fan_control_settings.clone() {
                    existing_settings.curve = curve;
                    Some(existing_settings)
                } else {
                    Some(FanControlSettings {
                        curve,
                        temperature_key: "edge".to_owned(),
                        interval_ms: 500,
                    })
                }
            }
            None => None,
        };

        self.edit_gpu_config(id.to_owned(), |config| {
            config.fan_control_enabled = enabled;
            config.fan_control_settings = settings;
        })
        .await
    }

    pub async fn set_power_cap(&'a self, id: &str, maybe_cap: Option<f64>) -> anyhow::Result<()> {
        self.edit_gpu_config(id.to_owned(), |gpu_config| {
            gpu_config.power_cap = maybe_cap;
        })
        .await
    }

    pub async fn set_performance_level(
        &self,
        id: &str,
        level: PerformanceLevel,
    ) -> anyhow::Result<()> {
        self.edit_gpu_config(id.to_owned(), |gpu_config| {
            gpu_config.performance_level = Some(level);
        })
        .await
    }

    pub async fn set_clocks_value(
        &self,
        id: &str,
        command: SetClocksCommand,
    ) -> anyhow::Result<()> {
        if let SetClocksCommand::Reset = command {
            self.controller_by_id(id)?.handle.reset_clocks_table()?;
        }

        self.edit_gpu_config(id.to_owned(), |gpu_config| match command {
            SetClocksCommand::MaxCoreClock(clock) => gpu_config.max_core_clock = Some(clock),
            SetClocksCommand::MaxMemoryClock(clock) => gpu_config.max_memory_clock = Some(clock),
            SetClocksCommand::MaxVoltage(voltage) => gpu_config.max_voltage = Some(voltage),
            SetClocksCommand::VoltageOffset(offset) => gpu_config.voltage_offset = Some(offset),
            SetClocksCommand::Reset => {
                gpu_config.max_core_clock = None;
                gpu_config.max_memory_clock = None;
                gpu_config.max_voltage = None;
            }
        })
        .await
    }

    pub async fn cleanup(self) {
        for (id, controller) in self.gpu_controllers.iter() {
            if controller.handle.get_clocks_table().is_ok() {
                if let Err(err) = controller.handle.reset_clocks_table() {
                    error!("Could not reset the clocks table: {err}");
                }
            }

            if let Err(err) = controller.apply_config(&config::Gpu::default()).await {
                error!("Could not reset settings for controller {id}: {err:#}");
            }
        }
    }
}
