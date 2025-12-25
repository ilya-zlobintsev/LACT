use super::gpu_controller::CommonControllerInfo;
use crate::server::gpu_controller::PciSlotInfo;
use anyhow::{bail, Context};
use lact_schema::OpenCLInfo;
use serde::Deserialize;
use tracing::error;

#[cfg_attr(test, allow(unused_variables))]
pub async fn get_opencl_info(info: &CommonControllerInfo, unique_vendor: bool) -> Vec<OpenCLInfo> {
    try_get_opencl_info(info, unique_vendor)
        .await
        .inspect_err(|err| {
            #[cfg(not(test))]
            tracing::warn!("could not fetch OpenCL info: {err:#}");
        })
        .unwrap_or_default()
}

#[cfg(not(test))]
async fn try_get_opencl_info(
    info: &CommonControllerInfo,
    unique_vendor: bool,
) -> anyhow::Result<Vec<OpenCLInfo>> {
    use tokio::process::Command;

    let clinfo_output = Command::new("clinfo")
        .arg("--json")
        .output()
        .await
        .context("Could not run 'clinfo'")?;

    if !clinfo_output.status.success() {
        bail!(
            "Exit code {} for 'clinfo': {} {}",
            clinfo_output.status,
            String::from_utf8_lossy(&clinfo_output.stdout),
            String::from_utf8_lossy(&clinfo_output.stderr)
        );
    }

    let cl_info: ClInfo<'_> =
        serde_json::from_slice(&clinfo_output.stdout).context("Could not parse 'clinfo' output")?;

    let expected_slot = info.get_slot_info()?;

    Ok(extract_device_info(
        &cl_info,
        info,
        &expected_slot,
        unique_vendor,
    ))
}

#[cfg(test)]
#[allow(clippy::unused_async)]
async fn try_get_opencl_info(
    info: &CommonControllerInfo,
    unique_vendor: bool,
) -> anyhow::Result<Vec<OpenCLInfo>> {
    let base_path = info
        .sysfs_path
        .parent()
        .and_then(|path| path.parent())
        .context("Could not get test parent path")?;

    let file_path = base_path.join("clinfo.json");
    if !file_path.exists() {
        bail!("'clinfo.json' not present in test data");
    }

    let data = std::fs::read_to_string(&file_path)?;
    let cl_info: ClInfo<'_> =
        serde_json::from_str(&data).context("Could not parse 'clinfo.json'")?;

    let expected_slot = info.get_slot_info()?;

    Ok(extract_device_info(
        &cl_info,
        info,
        &expected_slot,
        unique_vendor,
    ))
}

fn extract_device_info(
    cl_info: &ClInfo<'_>,
    device_info: &CommonControllerInfo,
    expected_slot: &PciSlotInfo,
    unique_vendor: bool,
) -> Vec<OpenCLInfo> {
    let mut devices = Vec::new();

    for (platform_i, platform_devices) in cl_info.devices.iter().enumerate() {
        for device in &platform_devices.online {
            for bus_info in [
                device.cl_device_pci_bus_info_khr,
                device.cl_device_topology_amd,
            ] {
                if bus_info
                    .and_then(parse_bus_info)
                    .is_some_and(|bus_info| bus_info == *expected_slot)
                {
                    let Some(platform) = cl_info.platforms.get(platform_i) else {
                        error!("Invalid clinfo platform index {platform_i}");
                        continue;
                    };

                    devices.push(make_opencl_info(platform, device));
                }
            }
        }
    }

    // If no devices were matched by the PCI slot id, get the first device with the matching vendor, as long as it is the only device with that vendor
    if unique_vendor && devices.is_empty() {
        let Ok(expected_vendor_id) =
            u32::from_str_radix(&device_info.pci_info.device_pci_info.vendor_id, 16)
        else {
            return vec![];
        };

        for (platform_i, platform_devices) in cl_info.devices.iter().enumerate() {
            for device in &platform_devices.online {
                if device.cl_device_vendor_id == expected_vendor_id {
                    let Some(platform) = cl_info.platforms.get(platform_i) else {
                        error!("Invalid clinfo platform index {platform_i}");
                        continue;
                    };

                    devices.push(make_opencl_info(platform, device));
                }
            }
        }
    }

    devices
}

fn make_opencl_info(platform: &ClPlatform, device: &ClDevice) -> OpenCLInfo {
    OpenCLInfo {
        platform_name: platform.cl_platform_name.to_owned(),
        device_name: device.cl_device_name.to_owned(),
        version: device.cl_device_version.to_owned(),
        driver_version: device.cl_driver_version.to_owned(),
        c_version: device.cl_device_opencl_c_version.to_owned(),
        compute_units: device.cl_device_max_compute_units,
        workgroup_size: device.cl_device_max_work_group_size,
        global_memory: device.cl_device_global_mem_size,
        local_memory: device.cl_device_local_mem_size,
    }
}

fn parse_bus_info(bus_info: &str) -> Option<PciSlotInfo> {
    let bus_info = bus_info.strip_prefix("PCI-E, ")?;
    let mut split = bus_info.split(':');

    let domain = u16::from_str_radix(split.next()?, 16).ok()?;
    let mut raw_bus = split.next()?;

    // Fix invalid ROCM bus format: `PCI-E, 0000:ffffffc1:00.0`
    if raw_bus.len() > 2 {
        raw_bus = &raw_bus[raw_bus.len() - 2..];
    }

    let bus = u16::from_str_radix(raw_bus, 16).ok()?;

    let full_dev = split.next()?;
    let dev_parts = full_dev.split_once('.')?;

    let dev = u16::from_str_radix(dev_parts.0, 16).ok()?;
    let func = u16::from_str_radix(dev_parts.1, 16).ok()?;

    Some(PciSlotInfo {
        domain,
        bus,
        dev,
        func,
    })
}

#[derive(Deserialize, PartialEq, Debug)]
struct ClInfo<'a> {
    #[serde(default, borrow)]
    platforms: Vec<ClPlatform<'a>>,
    #[serde(default, borrow)]
    devices: Vec<ClDeviceList<'a>>,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "UPPERCASE")]
struct ClPlatform<'a> {
    cl_platform_name: &'a str,
}

#[derive(Deserialize, PartialEq, Debug)]
struct ClDeviceList<'a> {
    #[serde(default, borrow)]
    online: Vec<ClDevice<'a>>,
}

#[allow(clippy::struct_field_names)]
#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "UPPERCASE")]
struct ClDevice<'a> {
    cl_device_name: &'a str,
    cl_device_vendor: &'a str,
    cl_device_vendor_id: u32,
    cl_device_version: &'a str,
    cl_driver_version: &'a str,
    cl_device_opencl_c_version: &'a str,
    cl_device_pci_bus_info_khr: Option<&'a str>,
    cl_device_topology_amd: Option<&'a str>,
    #[serde(default)]
    cl_device_max_compute_units: u32,
    #[serde(default)]
    cl_device_global_mem_size: u64,
    #[serde(default)]
    cl_device_local_mem_size: u64,
    #[serde(default)]
    cl_device_max_work_group_size: usize,
}

#[cfg(test)]
mod tests {
    use crate::server::{gpu_controller::PciSlotInfo, opencl::parse_bus_info};
    use pretty_assertions::assert_eq;

    #[test]
    fn parse_khr_bus() {
        assert_eq!(
            Some(PciSlotInfo {
                domain: 0,
                bus: 0xc1,
                dev: 0,
                func: 0
            }),
            parse_bus_info("PCI-E, 0000:c1:00.0")
        );
    }

    #[test]
    fn parse_amd_topo_bus() {
        assert_eq!(
            Some(PciSlotInfo {
                domain: 0,
                bus: 0xc1,
                dev: 0,
                func: 0
            }),
            parse_bus_info("PCI-E, 0000:ffffffc1:00.0")
        );
    }
}
