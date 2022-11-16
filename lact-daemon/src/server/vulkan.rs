use crate::fork::run_forked;
use lact_schema::VulkanInfo;
use vulkano::{
    instance::{Instance, InstanceCreateInfo},
    VulkanLibrary,
};

pub fn get_vulkan_info(vendor_id: &str, device_id: &str) -> anyhow::Result<VulkanInfo> {
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
                        supported_features: vulkano_struct_to_vec(device.supported_features()),
                        supported_extensions: vulkano_struct_to_vec(device.supported_extensions()),
                    };
                    return Ok(info);
                }
            }

            Err("Could not find a vulkan device with matching pci ids".to_owned())
        })
    }
}

fn vulkano_struct_to_vec<D: std::fmt::Debug>(data: D) -> Vec<String> {
    let output = format!("{data:?}");
    let trimmed_output = output.trim_start_matches('[').trim_end_matches(']');

    trimmed_output
        .split(',')
        .map(|s| s.trim().to_owned())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::vulkano_struct_to_vec;
    use vulkano::device::{DeviceExtensions, Features};

    #[test]
    fn features_to_vec() {
        let features = Features {
            geometry_shader: true,
            tessellation_shader: true,
            ..Features::empty()
        };
        let vec = vulkano_struct_to_vec(features);
        assert_eq!(vec, vec!["geometryShader", "tessellationShader"]);
    }

    #[test]
    fn extensions_to_vec() {
        let extensions = DeviceExtensions {
            khr_external_fence: true,
            khr_video_queue: true,
            ..DeviceExtensions::empty()
        };

        let vec = vulkano_struct_to_vec(extensions);
        assert_eq!(vec, vec!["VK_KHR_external_fence", "VK_KHR_video_queue"]);
    }
}
