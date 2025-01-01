#![allow(clippy::module_name_repetitions)]
mod amd;
pub mod fan_control;
mod intel;
mod nvidia;

pub use amd::AmdGpuController;
pub use intel::IntelGpuController;
pub use nvidia::NvidiaGpuController;

use crate::config::{self};
use amdgpu_sysfs::gpu_handle::power_profile_mode::PowerProfileModesTable;
use futures::future::LocalBoxFuture;
use lact_schema::{ClocksInfo, DeviceInfo, DeviceStats, GpuPciInfo, PowerStates};
use std::{path::Path, rc::Rc};
use tokio::{sync::Notify, task::JoinHandle};

type FanControlHandle = (Rc<Notify>, JoinHandle<()>);

pub trait GpuController {
    fn get_id(&self) -> anyhow::Result<String>;

    fn get_pci_info(&self) -> Option<&GpuPciInfo>;

    fn get_path(&self) -> &Path;

    fn get_info(&self) -> DeviceInfo;

    fn get_pci_slot_name(&self) -> Option<String>;

    fn apply_config<'a>(
        &'a self,
        config: &'a config::Gpu,
    ) -> LocalBoxFuture<'a, anyhow::Result<()>>;

    fn get_stats(&self, gpu_config: Option<&config::Gpu>) -> DeviceStats;

    fn get_clocks_info(&self) -> anyhow::Result<ClocksInfo>;

    fn get_power_states(&self, gpu_config: Option<&config::Gpu>) -> PowerStates;

    fn reset_pmfw_settings(&self);

    fn cleanup_clocks(&self) -> anyhow::Result<()>;

    fn get_power_profile_modes(&self) -> anyhow::Result<PowerProfileModesTable>;

    fn vbios_dump(&self) -> anyhow::Result<Vec<u8>>;
}
