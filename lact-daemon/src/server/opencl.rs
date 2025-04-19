use super::gpu_controller::{CommonControllerInfo, PciSlotInfo};
use anyhow::anyhow;
use cl3::{
    device,
    ext::{
        CL_DEVICE_GLOBAL_MEM_SIZE, CL_DEVICE_LOCAL_MEM_SIZE, CL_DEVICE_MAX_COMPUTE_UNITS,
        CL_DEVICE_NAME, CL_DEVICE_PCI_BUS_INFO_KHR, CL_DEVICE_TYPE_ALL, CL_DEVICE_VERSION,
        CL_PLATFORM_NAME,
    },
    platform,
};
use lact_schema::OpenCLInfo;
use std::ffi::c_void;
use tracing::error;

pub fn get_opencl_info(info: &CommonControllerInfo) -> Option<OpenCLInfo> {
    match try_get_opencl_info(info) {
        Ok(info) => info,
        Err(err) => {
            error!("could not get OpenCL info: {err}");
            None
        }
    }
}

fn try_get_opencl_info(info: &CommonControllerInfo) -> anyhow::Result<Option<OpenCLInfo>> {
    let slot_info = info.get_slot_info()?;

    let Some((platform, device)) = find_matching_device(&slot_info)? else {
        return Ok(None);
    };

    let platform_name = platform::get_platform_info(platform, CL_PLATFORM_NAME)
        .map_err(|err| anyhow!("Could not get platform name: {err}"))?
        .to_string()
        .replace('\0', "");

    let device_name = device::get_device_info(device, CL_DEVICE_NAME)
        .map_err(|err| anyhow!("Could not get device name: {err}"))?
        .to_string()
        .replace('\0', "");

    let version = device::get_device_info(device, CL_DEVICE_VERSION)
        .map_err(|err| anyhow!("Could not get device version: {err}"))?
        .to_string()
        .replace('\0', "");

    let compute_units = device::get_device_info(device, CL_DEVICE_MAX_COMPUTE_UNITS)
        .map_err(|err| anyhow!("Could not get device cu count: {err}"))?
        .to_uint();

    let global_memory = device::get_device_info(device, CL_DEVICE_GLOBAL_MEM_SIZE)
        .map_err(|err| anyhow!("Could not get device memory: {err}"))?
        .to_ulong();

    let local_memory = device::get_device_info(device, CL_DEVICE_LOCAL_MEM_SIZE)
        .map_err(|err| anyhow!("Could not get device memory: {err}"))?
        .to_ulong();

    Ok(Some(OpenCLInfo {
        platform_name,
        device_name,
        version,
        compute_units,
        global_memory,
        local_memory,
    }))
}

fn find_matching_device(
    slot_info: &PciSlotInfo,
) -> anyhow::Result<Option<(*mut c_void, *mut c_void)>> {
    let platforms = platform::get_platform_ids()
        .map_err(|err| anyhow!("Could not get platform list: {err}"))?;

    for platform in platforms {
        let devices = device::get_device_ids(platform, CL_DEVICE_TYPE_ALL)
            .map_err(|err| anyhow!("Could not get device list: {err}"))?;
        for device in devices {
            let raw_bus_info = device::get_device_info(device, CL_DEVICE_PCI_BUS_INFO_KHR)
                .map_err(|err| anyhow!("Could not get bus info: {err}"))?
                .to_vec_uchar();
            let bus_info = device::get_device_pci_bus_info_khr(&raw_bus_info);

            if bus_info.pci_bus == u32::from(slot_info.bus)
                && bus_info.pci_device == u32::from(slot_info.dev)
                && bus_info.pci_domain == u32::from(slot_info.domain)
                && bus_info.pci_function == u32::from(slot_info.func)
            {
                return Ok(Some((platform, device)));
            }
        }
    }

    Ok(None)
}
