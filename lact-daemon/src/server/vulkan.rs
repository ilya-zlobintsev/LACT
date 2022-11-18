use std::borrow::Cow;

use crate::fork::run_forked;
use lact_schema::VulkanInfo;
use vulkano::{
    instance::{Instance, InstanceCreateInfo},
    VulkanLibrary,
};

pub fn get_vulkan_info<'a>(vendor_id: &'a str, device_id: &'a str) -> anyhow::Result<VulkanInfo> {
    let vendor_id = u32::from_str_radix(vendor_id, 16)?;
    let device_id = u32::from_str_radix(device_id, 16)?;

    unsafe {
        run_forked(|| {
            let library = VulkanLibrary::new().map_err(|err| err.to_string())?;
            let instance = Instance::new(library, InstanceCreateInfo::default())
                .map_err(|err| err.to_string())?;
            let devices = instance
                .enumerate_physical_devices()
                .map_err(|err| err.to_string())?;

            for device in devices {
                let properties = device.properties();
                // Not sure how this works with systems that have multiple identical GPUs
                if properties.vendor_id == vendor_id && properties.device_id == device_id {
                    let info = VulkanInfo {
                        device_name: properties.device_name.clone(),
                        api_version: device.api_version().to_string(),
                        driver_name: properties.driver_name.clone(),
                        supported_features: device
                            .supported_features()
                            .into_iter()
                            .map(|(name, enabled)| (Cow::Borrowed(name), enabled))
                            .collect(),
                        supported_extensions: device
                            .supported_extensions()
                            .into_iter()
                            .map(|(name, enabled)| (Cow::Borrowed(name), enabled))
                            .collect(),
                    };
                    return Ok(info);
                }
            }

            Err("Could not find a vulkan device with matching pci ids".to_owned())
        })
    }
}
