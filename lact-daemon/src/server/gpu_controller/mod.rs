#![allow(clippy::module_name_repetitions)]
mod amd;
pub mod fan_control;
mod intel;
mod nvidia;

use amd::AmdGpuController;
use intel::IntelGpuController;
use nvidia::NvidiaGpuController;
use tracing::{error, info, warn};

use crate::config::{self};
use amdgpu_sysfs::gpu_handle::power_profile_mode::PowerProfileModesTable;
use anyhow::Context;
use futures::future::LocalBoxFuture;
use lact_schema::{ClocksInfo, DeviceInfo, DeviceStats, GpuPciInfo, PciInfo, PowerStates};
use nvml_wrapper::Nvml;
use std::{cell::OnceCell, collections::HashMap, fs, path::PathBuf, rc::Rc};
use tokio::{sync::Notify, task::JoinHandle};

type FanControlHandle = (Rc<Notify>, JoinHandle<()>);

pub trait GpuController {
    fn controller_info(&self) -> &CommonControllerInfo;

    fn get_info(&self) -> DeviceInfo;

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

#[derive(Clone)]
pub(crate) struct CommonControllerInfo {
    pub sysfs_path: PathBuf,
    pub pci_info: GpuPciInfo,
    pub pci_slot_name: String,
    pub driver: String,
}

impl CommonControllerInfo {
    pub fn build_id(&self) -> String {
        let GpuPciInfo {
            device_pci_info,
            subsystem_pci_info,
        } = &self.pci_info;

        format!(
            "{}:{}-{}:{}-{}",
            device_pci_info.vendor_id,
            device_pci_info.model_id,
            subsystem_pci_info.vendor_id,
            subsystem_pci_info.model_id,
            self.pci_slot_name
        )
    }
}

pub(crate) fn init_controller(
    path: PathBuf,
    pci_db: &pciid_parser::Database,
    nvml: &OnceCell<Option<Rc<Nvml>>>,
) -> anyhow::Result<Box<dyn GpuController>> {
    let uevent_path = path.join("uevent");
    let uevent = fs::read_to_string(uevent_path).context("Could not read 'uevent'")?;
    let mut uevent_map = parse_uevent(&uevent);

    let driver = uevent_map
        .remove("DRIVER")
        .context("DRIVER entry missing in 'uevent'")?
        .to_owned();
    let pci_slot_name = uevent_map
        .remove("PCI_SLOT_NAME")
        .context("PCI_SLOT_NAME entry missing in 'uevent'")?
        .to_owned();

    let (vendor_id, device_id) = uevent_map
        .get("PCI_ID")
        .and_then(|id_line| id_line.split_once(':'))
        .context("PCI_ID entry missing in 'uevent'")?;

    let subsystem_entry = uevent_map
        .get("PCI_SUBSYS_ID")
        .and_then(|id_line| id_line.split_once(':'));

    let (subsystem_vendor_id, subsystem_device_id) = subsystem_entry
        .map(|(vendor, device)| (Some(vendor), Some(device)))
        .unwrap_or_default();

    let subsystem_info = subsystem_entry
        .map(|(subsys_vendor_id, subsys_device_id)| {
            pci_db.get_device_info(vendor_id, device_id, subsys_vendor_id, subsys_device_id)
        })
        .unwrap_or_default();

    let vendor_entry = pci_db.vendors.get_key_value(vendor_id);

    let pci_info = GpuPciInfo {
        device_pci_info: PciInfo {
            vendor_id: vendor_id.to_owned(),
            vendor: vendor_entry.map(|(vendor_name, _)| vendor_name.clone()),
            model_id: device_id.to_owned(),
            model: vendor_entry.and_then(|(_, vendor)| {
                vendor
                    .devices
                    .get(device_id)
                    .map(|device| device.name.clone())
            }),
        },
        subsystem_pci_info: PciInfo {
            vendor_id: subsystem_vendor_id.unwrap_or_default().to_owned(),
            vendor: subsystem_info.subvendor_name.map(str::to_owned),
            model_id: subsystem_device_id.unwrap_or_default().to_owned(),
            model: subsystem_info.subdevice_name.map(str::to_owned),
        },
    };

    let common = CommonControllerInfo {
        sysfs_path: path,
        pci_info,
        pci_slot_name,
        driver,
    };

    match common.driver.as_str() {
        "amdgpu" | "radeon" => match AmdGpuController::new_from_path(common.clone()) {
            Ok(controller) => return Ok(Box::new(controller)),
            Err(err) => error!("could not initialize AMD controller: {err:#}"),
        },
        "i915" | "xe" => match IntelGpuController::new(common.clone()) {
            Ok(controller) => return Ok(Box::new(controller)),
            Err(err) => error!("could not initialize Intel controller: {err:#}"),
        },
        "nvidia" => {
            let nvml = nvml.get_or_init(|| match Nvml::init() {
                Ok(nvml) => {
                    info!("Nvidia management library loaded");
                    Some(Rc::new(nvml))
                }
                Err(err) => {
                    error!("could not load Nvidia management library: {err}, Nvidia controls will not be available");
                    None
                }
            });
            if let Some(nvml) = nvml {
                match NvidiaGpuController::new(common.clone(), nvml.clone()) {
                    Ok(controller) => {
                        return Ok(Box::new(controller));
                    }
                    Err(err) => error!("could not initialize Nvidia controller: {err:#}"),
                }
            }
        }
        _ => {
            warn!(
                "GPU at '{}' has unsupported driver '{}', functionality will be limited",
                common.sysfs_path.display(),
                common.driver,
            );
        }
    }

    // We use the AMD controller as the fallback even for non-AMD devices, it will at least
    // display basic device information from the SysFS
    Ok(Box::new(
        AmdGpuController::new_from_path(common).context("Could initialize fallback controller")?,
    ))
}

fn parse_uevent(data: &str) -> HashMap<&str, &str> {
    data.lines()
        .filter_map(|line| line.split_once('='))
        .collect()
}
