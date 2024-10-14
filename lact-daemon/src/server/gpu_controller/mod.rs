#![allow(clippy::module_name_repetitions)]
mod amd;
pub mod fan_control;

pub use amd::AmdGpuController;

use crate::config::{self};
use amdgpu_sysfs::gpu_handle::power_profile_mode::PowerProfileModesTable;
use amdgpu_sysfs::hw_mon::HwMon;
use futures::future::LocalBoxFuture;
use lact_schema::{
    ClocksInfo, DeviceInfo, DeviceStats, DrmInfo, GpuPciInfo, LinkInfo, PowerStates,
};
use std::collections::BTreeMap;
use std::{
    path::{Path, PathBuf},
    rc::Rc,
};
use tokio::{sync::Notify, task::JoinHandle};

type FanControlHandle = (Rc<Notify>, JoinHandle<()>);

pub trait GpuController {
    fn get_id(&self) -> anyhow::Result<String>;

    fn get_pci_info(&self) -> Option<&GpuPciInfo>;

    fn get_path(&self) -> &Path;

    fn get_info(&self) -> DeviceInfo;

    fn hw_monitors(&self) -> &[HwMon];

    fn get_full_vbios_version(&self) -> Option<String>;

    fn get_drm_info(&self) -> Option<DrmInfo>;

    fn get_pci_slot_name(&self) -> Option<String>;

    fn get_current_gfxclk(&self) -> Option<u16>;

    fn get_link_info(&self) -> LinkInfo;

    fn get_stats(&self, gpu_config: Option<&config::Gpu>) -> DeviceStats;

    fn get_throttle_info(&self) -> Option<BTreeMap<String, Vec<String>>>;

    fn get_clocks_info(&self) -> anyhow::Result<ClocksInfo>;

    fn get_power_states(&self, gpu_config: Option<&config::Gpu>) -> PowerStates;

    fn get_power_profile_modes(&self) -> anyhow::Result<PowerProfileModesTable>;

    fn reset_pmfw_settings(&self);

    fn vbios_dump(&self) -> anyhow::Result<Vec<u8>>;

    fn debugfs_path(&self) -> Option<PathBuf>;

    fn apply_config<'a>(
        &'a self,
        config: &'a config::Gpu,
    ) -> LocalBoxFuture<'a, anyhow::Result<()>>;

    fn cleanup_clocks(&self) -> anyhow::Result<()>;
}
