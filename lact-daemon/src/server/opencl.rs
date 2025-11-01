use super::gpu_controller::CommonControllerInfo;
use anyhow::anyhow;
use cl3::{
    device,
    ext::{
        cl_device_info, CL_DEVICE_GLOBAL_MEM_SIZE, CL_DEVICE_LOCAL_MEM_SIZE,
        CL_DEVICE_MAX_COMPUTE_UNITS, CL_DEVICE_MAX_WORK_GROUP_SIZE, CL_DEVICE_NAME,
        CL_DEVICE_OPENCL_C_VERSION, CL_DEVICE_PCI_BUS_INFO_KHR, CL_DEVICE_TOPOLOGY_AMD,
        CL_DEVICE_TYPE_ALL, CL_DEVICE_VENDOR_ID, CL_DEVICE_VERSION, CL_DRIVER_VERSION,
        CL_PLATFORM_NAME,
    },
    platform,
};
use lact_schema::OpenCLInfo;
use std::{collections::BTreeMap, ffi::c_void};
use tracing::{debug, error};

#[cfg_attr(test, allow(unreachable_code, unused_variables))]
pub fn get_opencl_info(info: &CommonControllerInfo, unique_vendor: bool) -> Option<OpenCLInfo> {
    #[cfg(test)]
    return None;

    match try_get_opencl_info(info, unique_vendor) {
        Ok(info) => info,
        Err(err) => {
            error!("could not get OpenCL info: {err}");
            None
        }
    }
}

fn try_get_opencl_info(
    info: &CommonControllerInfo,
    unique_vendor: bool,
) -> anyhow::Result<Option<OpenCLInfo>> {
    let Some((platform, device)) = find_matching_device(info, unique_vendor)? else {
        return Ok(None);
    };

    let platform_name = platform::get_platform_info(platform, CL_PLATFORM_NAME)
        .map_err(|err| anyhow!("Could not get platform name: {err}"))?
        .to_string()
        .replace('\0', "");

    let device_name = get_info_string(device, CL_DEVICE_NAME)?;
    let version = get_info_string(device, CL_DEVICE_VERSION)?;
    let driver_version = get_info_string(device, CL_DRIVER_VERSION)?;
    let c_version = get_info_string(device, CL_DEVICE_OPENCL_C_VERSION)?;

    let compute_units = device::get_device_info(device, CL_DEVICE_MAX_COMPUTE_UNITS)
        .map_err(|err| anyhow!("Could not get device cu count: {err}"))?
        .to_uint();

    let workgroup_size = device::get_device_info(device, CL_DEVICE_MAX_WORK_GROUP_SIZE)
        .map_err(|err| anyhow!("Could not get device cu count: {err}"))?
        .to_size();

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
        driver_version,
        c_version,
        workgroup_size,
        compute_units,
        global_memory,
        local_memory,
    }))
}

fn find_matching_device(
    info: &CommonControllerInfo,
    unique_vendor: bool,
) -> anyhow::Result<Option<(*mut c_void, *mut c_void)>> {
    let slot_info = info.get_slot_info()?;

    let platforms = platform::get_platform_ids()
        .map_err(|err| anyhow!("Could not get platform list: {err}"))?;

    let platform_devices = platforms
        .into_iter()
        .map(|platform| {
            let devices = device::get_device_ids(platform, CL_DEVICE_TYPE_ALL)
                .map_err(|err| anyhow!("Could not get device list: {err}"))?;
            anyhow::Ok((platform, devices))
        })
        .collect::<anyhow::Result<BTreeMap<_, _>>>()?;

    for (platform, devices) in &platform_devices {
        for device in devices {
            if let Ok(raw_amd_topology) = device::get_device_info(*device, CL_DEVICE_TOPOLOGY_AMD) {
                let amd_topology =
                    device::get_amd_device_topology(&raw_amd_topology.to_vec_uchar());

                if u16::from(amd_topology.bus) == slot_info.bus
                    && u16::from(amd_topology.device) == slot_info.dev
                    && u16::from(amd_topology.function) == slot_info.func
                {
                    return Ok(Some((*platform, *device)));
                }
            }

            if let Ok(raw_bus_info) = device::get_device_info(*device, CL_DEVICE_PCI_BUS_INFO_KHR)
                .map_err(|err| anyhow!("Could not get bus info: {err}"))
            {
                let bus_info = device::get_device_pci_bus_info_khr(&raw_bus_info.to_vec_uchar());
                if bus_info.pci_bus == u32::from(slot_info.bus)
                    && bus_info.pci_device == u32::from(slot_info.dev)
                    && bus_info.pci_domain == u32::from(slot_info.domain)
                    && bus_info.pci_function == u32::from(slot_info.func)
                {
                    return Ok(Some((*platform, *device)));
                }
            }
        }
    }

    // If no devices were matched by the PCI slot id, get the first device with the matching vendor, as long as it is the only device with that vendor
    if unique_vendor {
        let expected_vendor_id = u32::from_str_radix(&info.pci_info.device_pci_info.vendor_id, 16)?;

        for (platform, devices) in platform_devices {
            for device in devices {
                if let Ok(raw_vendor_id) = device::get_device_info(device, CL_DEVICE_VENDOR_ID)
                    .map_err(|err| anyhow!("Could not get bus info: {err}"))
                {
                    if raw_vendor_id.to_uint() == expected_vendor_id {
                        debug!("found matching OpenCL device with vendor check fallback");
                        return Ok(Some((platform, device)));
                    }
                }
            }
        }
    }

    Ok(None)
}

fn get_info_string(device: *mut c_void, param: cl_device_info) -> anyhow::Result<String> {
    let mut string = device::get_device_info(device, param)
        .map_err(|err| anyhow!("Could not fetch property {param:0x}: {err}"))?
        .to_string();
    string.pop();
    Ok(string)
}
