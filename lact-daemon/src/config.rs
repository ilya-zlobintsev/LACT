use crate::server::gpu_controller::{GpuController, VENDOR_NVIDIA};
use anyhow::Context;
use indexmap::IndexMap;
use lact_schema::config::{GpuConfig, Profile};
use nix::unistd::{getuid, Group};
use notify::{RecommendedWatcher, Watcher};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::{
    cell::Cell,
    collections::BTreeMap,
    env, fs, iter,
    path::PathBuf,
    rc::Rc,
    time::{Duration, Instant},
};
use tokio::{sync::mpsc, time};
use tracing::{debug, error, info};

const FILE_NAME: &str = "config.yaml";
const DEFAULT_ADMIN_GROUPS: [&str; 2] = ["wheel", "sudo"];
/// Minimum amount of time between separate config reloads
const CONFIG_RELOAD_INTERVAL_MILLIS: u64 = 50;
/// Period when config changes are ignored after LACT itself has edited the config
const SELF_CONFIG_EDIT_PERIOD_MILLIS: u64 = 1000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Config {
    #[serde(default)]
    pub version: u64,
    pub daemon: Daemon,
    #[serde(default = "default_apply_settings_timer")]
    pub apply_settings_timer: u64,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    gpus: IndexMap<String, GpuConfig>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub profiles: IndexMap<Rc<str>, Profile>,
    #[serde(default)]
    pub current_profile: Option<Rc<str>>,
    #[serde(default)]
    pub auto_switch_profiles: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            daemon: Daemon::default(),
            apply_settings_timer: default_apply_settings_timer(),
            gpus: IndexMap::new(),
            profiles: IndexMap::new(),
            current_profile: None,
            auto_switch_profiles: false,
            version: 5,
        }
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Daemon {
    pub log_level: String,
    #[deprecated]
    #[serde(default, skip_serializing)]
    pub admin_groups: Vec<String>,
    pub admin_user: Option<String>,
    pub admin_group: Option<String>,
    #[serde(default)]
    pub disable_clocks_cleanup: bool,
    pub tcp_listen_address: Option<String>,
}

impl Default for Daemon {
    fn default() -> Self {
        let admin_user = env::var("FLATPAK_INSTALL_USER")
            .ok()
            .filter(|user| !user.is_empty());

        #[allow(deprecated)]
        Self {
            log_level: "info".to_owned(),
            admin_user,
            admin_group: find_existing_group(&DEFAULT_ADMIN_GROUPS),
            admin_groups: vec![],
            disable_clocks_cleanup: false,
            tcp_listen_address: None,
        }
    }
}

impl Config {
    pub fn load() -> anyhow::Result<Option<Self>> {
        let path = get_path(FILE_NAME);
        if path.exists() {
            let raw_config = fs::read_to_string(path).context("Could not open config file")?;
            let config =
                serde_yaml::from_str(&raw_config).context("Could not deserialize config")?;
            Ok(Some(config))
        } else {
            let parent = path.parent().unwrap();
            fs::create_dir_all(parent)?;
            Ok(None)
        }
    }

    pub fn save(&self, config_last_saved: &Cell<Instant>) -> anyhow::Result<()> {
        self.save_with_name(config_last_saved, FILE_NAME)
    }

    #[allow(clippy::pedantic)]
    pub fn save_with_name(
        &self,
        config_last_saved: &Cell<Instant>,
        filename: &str,
    ) -> anyhow::Result<()> {
        let path = get_path(filename);
        debug!("saving config to {path:?}");

        #[cfg(not(test))]
        {
            let raw_config = serde_yaml::to_string(self)?;
            fs::write(path, raw_config).context("Could not write config")?;
        }

        config_last_saved.set(Instant::now());

        Ok(())
    }

    pub fn load_or_create() -> anyhow::Result<Self> {
        if let Some(config) = Config::load()? {
            Ok(config)
        } else {
            let config = Config::default();
            config.save(&Cell::new(Instant::now()))?;
            Ok(config)
        }
    }

    #[allow(clippy::cast_precision_loss)]
    pub fn migrate_versions(&mut self, gpu_controllers: &BTreeMap<String, Box<dyn GpuController>>) {
        loop {
            let next_version = self.version + 1;

            let gpu_configs = self.gpus.iter_mut().chain(
                self.profiles
                    .values_mut()
                    .flat_map(|profile| profile.gpus.iter_mut()),
            );

            match next_version {
                0 => unreachable!(),
                // Reset VRAM settings on Nvidia after new offset ratio logic
                1 => {
                    for (id, gpu) in gpu_configs {
                        if id.starts_with(VENDOR_NVIDIA) {
                            gpu.clocks_configuration.max_memory_clock = None;
                            gpu.clocks_configuration.min_memory_clock = None;
                        }
                    }
                }
                2 => {
                    for (id, gpu) in gpu_configs {
                        if id.starts_with(VENDOR_NVIDIA) {
                            gpu.clocks_configuration.max_core_clock = None;
                            gpu.clocks_configuration.max_memory_clock = None;
                        }
                    }
                }
                3 => {
                    for (id, gpu) in gpu_configs {
                        if let Some(controller) = gpu_controllers.get(id) {
                            let stats = controller.get_stats(Some(gpu));

                            if let Some(fan_settings) = &mut gpu.fan_control_settings {
                                if let (Some(pwm_min), Some(pwm_max)) =
                                    (stats.fan.pwm_min, stats.fan.pwm_max)
                                {
                                    let ratio_min = (pwm_min as f32) / f32::from(u8::MAX);
                                    let ratio_max = (pwm_max as f32) / f32::from(u8::MAX);

                                    for value in fan_settings
                                        .curve
                                        .0
                                        .values_mut()
                                        .chain(iter::once(&mut (fan_settings.static_speed)))
                                    {
                                        let mut updated_value = None;
                                        if *value < ratio_min {
                                            updated_value = Some(ratio_min);
                                        }
                                        if *value > ratio_max {
                                            updated_value = Some(ratio_max);
                                        }

                                        if let Some(new_value) = updated_value {
                                            let new_value = (new_value * 100.0).round() / 100.0;
                                            info!("updated fan curve speed point {}% to {}% to be within the allowed range", *value * 100.0, new_value * 100.0);
                                            *value = new_value;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                #[allow(deprecated)]
                4 => {
                    self.daemon.admin_group = find_existing_group(&self.daemon.admin_groups);
                    self.daemon.admin_groups.clear();
                }
                5 => {
                    if let Ok(admin_user) = env::var("FLATPAK_INSTALL_USER") {
                        if self.daemon.admin_user.is_none() {
                            self.daemon.admin_user = Some(admin_user);
                        }
                    }
                }
                _ => break,
            }
            info!("migrated config version {} to {next_version}", self.version);
            self.version = next_version;
        }
    }

    /// Gets the GPU configs according to the current profile. Returns an error if the current profile could not be found.
    pub fn gpus(&self) -> anyhow::Result<&IndexMap<String, GpuConfig>> {
        match &self.current_profile {
            Some(profile) => {
                let profile = self
                    .profiles
                    .get(profile)
                    .with_context(|| format!("Could not find profile '{profile}'"))?;
                Ok(&profile.gpus)
            }
            None => Ok(&self.gpus),
        }
    }

    /// Same as [`gpus`], but with a mutable reference
    pub fn gpus_mut(&mut self) -> anyhow::Result<&mut IndexMap<String, GpuConfig>> {
        match &self.current_profile {
            Some(profile) => {
                let profile = self
                    .profiles
                    .get_mut(profile)
                    .with_context(|| format!("Could not find profile '{profile}'"))?;
                Ok(&mut profile.gpus)
            }
            None => Ok(&mut self.gpus),
        }
    }

    /// Get a specific profile
    pub fn profile(&self, profile: &str) -> anyhow::Result<&Profile> {
        self.profiles
            .get(profile)
            .with_context(|| format!("Profile {profile} not found"))
    }

    /// Get the settings for "default" profile (aka no profile)
    pub fn default_profile(&self) -> Profile {
        Profile {
            gpus: self.gpus.clone(),
            rule: None,
        }
    }

    pub fn clear(&mut self) {
        self.gpus.clear();
        self.profiles.clear();
        self.current_profile = None;
    }
}

pub fn start_watcher(config_last_saved: Rc<Cell<Instant>>) -> mpsc::UnboundedReceiver<Config> {
    let (config_tx, config_rx) = mpsc::unbounded_channel();
    let (event_tx, mut event_rx) = mpsc::channel(64);

    tokio::task::spawn_local(async move {
        let mut watcher =
            RecommendedWatcher::new(SenderEventHandler(event_tx), notify::Config::default())
                .expect("Could not create config file watcher");

        let config_path = get_path(FILE_NAME);
        let watch_path = config_path
            .parent()
            .expect("Config path always has a parent");
        watcher
            .watch(watch_path, notify::RecursiveMode::Recursive)
            .expect("Could not subscribe to config file changes");

        while let Some(res) = event_rx.recv().await {
            debug!("got config file event {res:?}");
            match res {
                Ok(event) => {
                    use notify::EventKind;

                    if let EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_) =
                        event.kind
                    {
                        if config_last_saved.get().elapsed()
                            < Duration::from_millis(SELF_CONFIG_EDIT_PERIOD_MILLIS)
                        {
                            debug!("ignoring fs event after self-inflicted config change");
                            continue;
                        }

                        // Accumulate FS events, reload config only after a period has passed since the last event
                        debug!(
                            "waiting for {CONFIG_RELOAD_INTERVAL_MILLIS}ms before reloading config"
                        );
                        let timeout =
                            time::sleep(Duration::from_millis(CONFIG_RELOAD_INTERVAL_MILLIS));
                        tokio::pin!(timeout);

                        loop {
                            tokio::select! {
                               () = &mut timeout => {
                                   break;
                               }
                               Some(res) = event_rx.recv() => {
                                    match res {
                                        Ok(event) => {
                                            if let EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_) = event.kind {
                                                debug!("got another fs event, resetting reload timer");
                                                timeout.as_mut().reset(time::Instant::now() + Duration::from_millis(CONFIG_RELOAD_INTERVAL_MILLIS));
                                            }
                                        }
                                        Err(err) => error!("filesystem event error: {err}")
                                    }
                               }
                            }
                        }

                        match Config::load() {
                            Ok(Some(new_config)) => config_tx.send(new_config).unwrap(),
                            Ok(None) => error!("config was removed!"),
                            Err(err) => {
                                error!("could not read config after it was changed: {err:#}");
                            }
                        }
                    }
                }
                Err(err) => error!("filesystem event error: {err}"),
            }
        }

        debug!("registered config file event listener at path {watch_path:?}");
    });

    config_rx
}

struct SenderEventHandler(mpsc::Sender<notify::Result<notify::Event>>);

impl notify::EventHandler for SenderEventHandler {
    fn handle_event(&mut self, event: notify::Result<notify::Event>) {
        let _ = self.0.blocking_send(event);
    }
}

fn get_path(filename: &str) -> PathBuf {
    let uid = getuid();
    if uid.is_root() {
        PathBuf::from("/etc/lact").join(filename)
    } else {
        let config_dir = PathBuf::from(env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
            let home = env::var("HOME").expect("$HOME variable is not set");
            format!("{home}/.config")
        }));
        config_dir.join("lact").join(filename)
    }
}

fn default_apply_settings_timer() -> u64 {
    5
}

fn find_existing_group(groups: &[impl AsRef<str>]) -> Option<String> {
    groups
        .iter()
        .find_map(|group_name| Group::from_name(group_name.as_ref()).ok().flatten())
        .map(|group| group.name)
}

#[cfg(test)]
mod tests {
    use crate::config::{Config, Daemon};
    use indexmap::IndexMap;
    use insta::assert_yaml_snapshot;
    use lact_schema::{
        config::{ClocksConfiguration, FanControlSettings, FanCurve, GpuConfig},
        FanControlMode, PmfwOptions,
    };
    use std::collections::BTreeMap;

    #[test]
    fn serde_de_full() {
        let config = Config {
            daemon: Daemon::default(),
            gpus: [(
                "my-gpu-id".to_owned(),
                GpuConfig {
                    fan_control_enabled: true,
                    fan_control_settings: Some(FanControlSettings {
                        curve: FanCurve::default(),
                        temperature_key: "edge".to_owned(),
                        interval_ms: 500,
                        mode: FanControlMode::Curve,
                        static_speed: 0.5,
                        spindown_delay_ms: Some(5000),
                        change_threshold: Some(3),
                        auto_threshold: Some(40),
                    }),
                    ..Default::default()
                },
            )]
            .into(),
            ..Default::default()
        };
        let data = serde_yaml::to_string(&config).unwrap();
        let deserialized_config: Config = serde_yaml::from_str(&data).unwrap();
        assert_eq!(config, deserialized_config);
    }

    #[test]
    fn parse_doc() {
        let doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../docs/CONFIG.md"));
        let example_config_start = doc
            .find("```yaml")
            .expect("Could not find example config start")
            + 7;
        let example_config_end = doc[example_config_start..]
            .find("```")
            .expect("Could not find example config end")
            + example_config_start;
        let example_config = &doc[example_config_start..example_config_end];

        let deserialized_config: Config = serde_yaml::from_str(example_config).unwrap();
        assert_yaml_snapshot!(deserialized_config);
    }

    #[test]
    fn clocks_configuration_applied() {
        let mut gpu = GpuConfig {
            fan_control_enabled: false,
            fan_control_settings: None,
            pmfw_options: PmfwOptions::default(),
            power_cap: None,
            performance_level: None,
            clocks_configuration: ClocksConfiguration::default(),
            power_profile_mode_index: None,
            custom_power_profile_mode_hueristics: vec![],
            power_states: IndexMap::new(),
        };

        assert!(!gpu.is_core_clocks_used());
        gpu.clocks_configuration.voltage_offset = Some(10);
        assert!(gpu.is_core_clocks_used());
    }

    #[test]
    fn migrate_versions() {
        let mut config = Config {
            version: 0,
            daemon: Daemon::default(),
            apply_settings_timer: 5,
            gpus: IndexMap::from([
                (
                    "10DE:2704-1462:5110-0000:09:00.0".to_owned(),
                    GpuConfig {
                        clocks_configuration: ClocksConfiguration {
                            max_core_clock: Some(3000),
                            max_memory_clock: Some(10_000),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                ),
                (
                    "1002:687F-1043:0555-0000:0b:00.0".to_owned(),
                    GpuConfig {
                        clocks_configuration: ClocksConfiguration {
                            max_core_clock: Some(1500),
                            max_memory_clock: Some(920),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                ),
            ]),
            profiles: IndexMap::new(),
            current_profile: None,
            auto_switch_profiles: false,
        };

        config.migrate_versions(&BTreeMap::new());

        assert_eq!(
            config
                .gpus
                .get("10DE:2704-1462:5110-0000:09:00.0")
                .unwrap()
                .clocks_configuration
                .max_core_clock,
            None,
        );
        assert_eq!(
            config
                .gpus
                .get("10DE:2704-1462:5110-0000:09:00.0")
                .unwrap()
                .clocks_configuration
                .max_memory_clock,
            None,
        );
        assert_eq!(
            config
                .gpus
                .get("1002:687F-1043:0555-0000:0b:00.0")
                .unwrap()
                .clocks_configuration
                .max_memory_clock,
            Some(920),
        );
    }
}
