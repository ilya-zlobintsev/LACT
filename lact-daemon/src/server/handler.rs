use super::gpu_controller::{fan_control::FanCurve, GpuController};
use crate::config::{Config, FanControlSettings, GpuConfig};
use amdgpu_sysfs::sysfs::SysFS;
use anyhow::{anyhow, Context};
use lact_schema::{ClocksInfo, DeviceInfo, DeviceListEntry, DeviceStats, FanCurveMap};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, RwLock},
    time::Duration,
};
use tracing::{debug, info, trace, warn};

#[derive(Clone)]
pub struct Handler {
    pub config: Arc<RwLock<Config>>,
    pub gpu_controllers: Arc<HashMap<String, GpuController>>,
}

impl<'a> Handler {
    pub async fn new(config: Config) -> anyhow::Result<Self> {
        let mut controllers = HashMap::new();

        let base_path = PathBuf::from("/sys/class/drm");

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
                    Ok(controller) => {
                        let handle = &controller.handle;
                        let pci_id = handle.get_pci_id().context("Device has no vendor id")?;
                        let pci_subsys_id = handle
                            .get_pci_subsys_id()
                            .context("Device has no subsys id")?;
                        let pci_slot_name = handle
                            .get_pci_slot_name()
                            .context("Device has no pci slot")?;

                        let id = format!(
                            "{}:{}-{}:{}-{}",
                            pci_id.0, pci_id.1, pci_subsys_id.0, pci_subsys_id.1, pci_slot_name
                        );

                        debug!(
                            "initialized GPU controller {} for path {:?}",
                            id,
                            handle.get_path()
                        );

                        controllers.insert(id, controller);
                    }
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
                if gpu_config.fan_control_enabled {
                    let settings = gpu_config.fan_control_settings.as_ref().context(
                        "Fan control is enabled but no settings are defined (invalid config?)",
                    )?;
                    let interval = Duration::from_millis(settings.interval_ms);
                    controller
                        .start_fan_control(
                            settings.curve.clone(),
                            settings.temperature_key.clone(),
                            interval,
                        )
                        .await?;
                }

                if let Some(power_cap) = gpu_config.power_cap {
                    controller
                        .handle
                        .hw_monitors
                        .first()
                        .context("GPU has power cap defined but has no hardware monitor")?
                        .set_power_cap(power_cap)
                        .context("Could not set power cap")?;
                }
            } else {
                info!("Could not find GPU with id {id} defined in configuration");
            }
        }

        Ok(Self {
            gpu_controllers: Arc::new(controllers),
            config: Arc::new(RwLock::new(config)),
        })
    }

    fn edit_config<F: FnOnce(&mut Config)>(&self, f: F) -> anyhow::Result<()> {
        let mut config_guard = self.config.write().map_err(|err| anyhow!("{err}"))?;
        f(&mut config_guard);
        config_guard.save()?;
        Ok(())
    }

    fn edit_gpu_config<F: FnOnce(&mut GpuConfig)>(&self, id: String, f: F) -> anyhow::Result<()> {
        self.edit_config(|config| {
            let gpu_config = config.gpus.entry(id).or_default();
            f(gpu_config);
        })
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
        self.controller_by_id(id)?.get_stats()
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
        let settings = if enabled {
            let settings = {
                let curve = curve.map_or_else(FanCurve::default, FanCurve);
                curve.validate()?;

                let mut config_guard = self.config.write().map_err(|err| anyhow!("{err}"))?;
                let gpu_config = config_guard.gpus.entry(id.to_owned()).or_default();

                if let Some(mut existing_settings) = gpu_config.fan_control_settings.clone() {
                    existing_settings.curve = curve;
                    existing_settings
                } else {
                    FanControlSettings {
                        curve,
                        temperature_key: "edge".to_owned(),
                        interval_ms: 500,
                    }
                }
            };
            let interval = Duration::from_millis(settings.interval_ms);

            self.controller_by_id(id)?
                .start_fan_control(
                    settings.curve.clone(),
                    settings.temperature_key.clone(),
                    interval,
                )
                .await?;
            Some(settings)
        } else {
            self.controller_by_id(id)?.stop_fan_control(true).await?;
            None
        };

        self.edit_gpu_config(id.to_owned(), |config| {
            config.fan_control_enabled = enabled;
            config.fan_control_settings = settings
        })
    }

    pub fn set_power_cap(&'a self, id: &str, maybe_cap: Option<f64>) -> anyhow::Result<()> {
        let hw_mon = self
            .controller_by_id(id)?
            .handle
            .hw_monitors
            .first()
            .context("GPU has no hardware monitor")?;

        let cap = match maybe_cap {
            Some(cap) => cap,
            None => hw_mon.get_power_cap_default()?,
        };
        hw_mon.set_power_cap(cap)?;

        self.edit_gpu_config(id.to_owned(), |gpu_config| {
            gpu_config.power_cap = maybe_cap;
        })
    }

    pub async fn cleanup(self) {
        let config = self.config.read().unwrap().clone();
        for (id, gpu_config) in config.gpus {
            if let Ok(controller) = self.controller_by_id(&id) {
                if gpu_config.fan_control_enabled {
                    debug!("Stopping fan control");
                    controller
                        .stop_fan_control(true)
                        .await
                        .expect("Could not stop fan control");
                }

                if let (Some(_), Some(hw_mon)) =
                    (gpu_config.power_cap, controller.handle.hw_monitors.first())
                {
                    if let Ok(default_cap) = hw_mon.get_power_cap_default() {
                        debug!("Setting power limit to default");
                        hw_mon
                            .set_power_cap(default_cap)
                            .expect("Could not set power cap to default");
                    }
                }
            }
        }
    }
}
