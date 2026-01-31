#![allow(clippy::module_name_repetitions)]
mod amd;
pub mod common;
mod intel;
#[cfg(feature = "nvidia")]
mod nvidia;

use amd::AmdGpuController;
use intel::IntelGpuController;
use lact_schema::DeviceType;
use lact_schema::ProcessList;
#[cfg(feature = "nvidia")]
use nvidia::NvidiaGpuController;

pub const VENDOR_AMD: &str = "1002";
pub const VENDOR_NVIDIA: &str = "10DE";

use crate::server::handler::{AMD_DRM, INTEL_DRM, NVML};
use amdgpu_sysfs::gpu_handle::power_profile_mode::PowerProfileModesTable;
use anyhow::Context;
use anyhow::anyhow;
use futures::{FutureExt, future::LocalBoxFuture};
use lact_schema::{
    ClocksInfo, DeviceInfo, DeviceStats, GpuPciInfo, PciInfo, PowerStates, config::GpuConfig,
};
use std::io;
#[cfg(feature = "nvidia")]
use std::sync::Arc;
use std::{collections::HashMap, fs, path::PathBuf, rc::Rc};
use tokio::{sync::Notify, task::JoinHandle};
use tracing::{error, warn};

#[cfg(feature = "nvidia")]
pub use nvidia::nvapi::NvApi;
#[cfg(feature = "nvidia")]
use nvml_wrapper::Nvml;

pub type DynGpuController = Box<dyn GpuController>;
type FanControlHandle = (Rc<Notify>, JoinHandle<()>);

pub trait GpuController {
    fn controller_info(&self) -> &CommonControllerInfo;

    fn device_type(&self) -> DeviceType;

    fn get_info(&self, unique_vendor: bool) -> LocalBoxFuture<'_, DeviceInfo>;

    fn friendly_name(&self) -> Option<String>;

    fn apply_config<'a>(&'a self, config: &'a GpuConfig) -> LocalBoxFuture<'a, anyhow::Result<()>>;

    fn get_stats(&self, gpu_config: Option<&GpuConfig>) -> DeviceStats;

    fn get_clocks_info(&self, gpu_config: Option<&GpuConfig>) -> anyhow::Result<ClocksInfo>;

    fn get_power_states(&self, gpu_config: Option<&GpuConfig>) -> PowerStates;

    fn reset_pmfw_settings(&self);

    fn cleanup(&self) -> LocalBoxFuture<'_, ()> {
        async {}.boxed_local()
    }

    fn reset_clocks(&self) -> anyhow::Result<()>;

    fn get_power_profile_modes(&self) -> anyhow::Result<PowerProfileModesTable>;

    fn vbios_dump(&self) -> anyhow::Result<Vec<u8>>;

    fn process_list(&self) -> anyhow::Result<ProcessList>;
}

#[derive(Clone, Debug)]
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

    pub fn get_slot_info(&self) -> anyhow::Result<PciSlotInfo> {
        let [domain, bus, dev, func] = self
            .pci_slot_name
            .split([':', '.'])
            .map(|part| u16::from_str_radix(part, 16).context("Could not parse pci slot name part"))
            .collect::<anyhow::Result<Vec<_>>>()?
            .try_into()
            .map_err(|err| anyhow!("Invalid pci slot name format {err:?}"))?;

        Ok(PciSlotInfo {
            domain,
            bus,
            dev,
            func,
        })
    }

    pub fn get_drm_render(&self) -> io::Result<PathBuf> {
        fs::canonicalize(format!(
            "/dev/dri/by-path/pci-{}-render",
            self.pci_slot_name
        ))
    }

    pub fn get_drm_card(&self) -> io::Result<PathBuf> {
        fs::canonicalize(format!("/dev/dri/by-path/pci-{}-card", self.pci_slot_name))
    }
}

#[derive(Debug, PartialEq)]
pub struct PciSlotInfo {
    pub domain: u16,
    pub bus: u16,
    pub dev: u16,
    pub func: u16,
}

#[cfg(feature = "nvidia")]
pub type NvidiaLibs = (Arc<Nvml>, Arc<Option<NvApi>>);
#[cfg(not(feature = "nvidia"))]
pub type NvidiaLibs = ();

pub(crate) fn init_controller(
    path: PathBuf,
    pci_db: &pciid_parser::Database,
) -> anyhow::Result<Box<dyn GpuController>> {
    #[cfg(not(feature = "nvidia"))]
    let _ = NVML;

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
        .and_then(|(subsys_vendor_id, subsys_device_id)| {
            Some(pci_db.get_device_info(
                u16::from_str_radix(vendor_id, 16).ok()?,
                u16::from_str_radix(device_id, 16).ok()?,
                u16::from_str_radix(subsys_vendor_id, 16).ok()?,
                u16::from_str_radix(subsys_device_id, 16).ok()?,
            ))
        })
        .unwrap_or_default();

    let vendor = pci_db.vendors.get(&u16::from_str_radix(vendor_id, 16)?);

    let mut pci_info = GpuPciInfo {
        device_pci_info: PciInfo {
            vendor_id: vendor_id.to_owned(),
            vendor: vendor.map(|vendor| vendor.name.clone()),
            model_id: device_id.to_owned(),
            model: vendor.and_then(|vendor| {
                vendor
                    .devices
                    .get(&u16::from_str_radix(device_id, 16).ok()?)
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
    pci_info.subsystem_pci_info.model =
        get_embedded_device_name(&pci_info).or(pci_info.subsystem_pci_info.model);

    let common = CommonControllerInfo {
        sysfs_path: path,
        pci_info,
        pci_slot_name,
        driver,
    };

    match common.driver.as_str() {
        "amdgpu" | "radeon" => {
            match AmdGpuController::new_from_path(common.clone(), AMD_DRM.as_ref()) {
                Ok(controller) => return Ok(Box::new(controller)),
                Err(err) => error!("could not initialize AMD controller: {err:#}"),
            }
        }
        "i915" | "xe" => {
            if let Some(drm) = INTEL_DRM.as_ref() {
                match IntelGpuController::new(common.clone(), drm) {
                    Ok(controller) => return Ok(Box::new(controller)),
                    Err(err) => error!("could not initialize Intel controller: {err:#}"),
                }
            } else {
                error!("Intel DRM library missing, Intel controls will not be available");
            }
        }
        #[cfg(feature = "nvidia")]
        "nvidia" => {
            if let Some((nvml, nvapi)) = NVML.as_ref() {
                match NvidiaGpuController::new(common.clone(), nvml, nvapi.as_ref().as_ref()) {
                    Ok(controller) => {
                        return Ok(Box::new(controller));
                    }
                    Err(err) => error!("could not initialize Nvidia controller: {err:#}"),
                }
            } else {
                #[cfg(not(test))]
                error!("NVML is missing, Nvidia controls will not be available");
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
        AmdGpuController::new_from_path(common, None)
            .context("Could initialize fallback controller")?,
    ))
}

fn parse_uevent(data: &str) -> HashMap<&str, &str> {
    data.lines()
        .filter_map(|line| line.split_once('='))
        .collect()
}

fn get_embedded_device_name(pci_info: &GpuPciInfo) -> Option<String> {
    const EXTRA_IDS: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../res/device_ids.json"
    ));
    let extra_ids: serde_json::Value =
        serde_json::from_str(EXTRA_IDS).expect("Could not parse embedded db");

    extra_ids
        .get(pci_info.device_pci_info.vendor_id.to_ascii_lowercase())
        .and_then(|vendor| vendor.get(pci_info.device_pci_info.model_id.to_ascii_lowercase()))
        .and_then(|device| device.get(pci_info.subsystem_pci_info.vendor_id.to_ascii_lowercase()))
        .and_then(|subsys_vendor| {
            subsys_vendor.get(pci_info.subsystem_pci_info.model_id.to_ascii_lowercase())
        })
        .and_then(|subsys_device| subsys_device.as_str())
        .map(str::to_owned)
}
