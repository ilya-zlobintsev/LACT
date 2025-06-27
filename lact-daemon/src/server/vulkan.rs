use anyhow::{anyhow, bail, Context};
use indexmap::{map::Entry, IndexMap};
use lact_schema::{VulkanDriverInfo, VulkanInfo};
use serde::Deserialize;
use std::{env, fs, path::Path};
use tokio::process::Command;
use tracing::{error, trace};

use crate::server::gpu_controller::{CommonControllerInfo, PciSlotInfo};

include!(concat!(env!("OUT_DIR"), "/vulkan_constants.rs"));

#[cfg_attr(test, allow(unreachable_code, unused_variables))]
pub async fn get_vulkan_info(info: &CommonControllerInfo) -> anyhow::Result<Vec<VulkanInfo>> {
    #[cfg(test)]
    return Ok(vec![]);

    let mut results = Vec::new();

    let pci_info = &info.pci_info;
    let pci_slot_info = info.get_slot_info()?;

    trace!("Reading vulkan info");
    let vendor_id = u32::from_str_radix(&pci_info.device_pci_info.vendor_id, 16)?;
    let device_id = u32::from_str_radix(&pci_info.device_pci_info.model_id, 16)?;
    trace!("Reading vulkan info");

    let summary_output = vulkaninfo_command()
        .arg("--summary")
        .output()
        .await
        .context("Could not run 'vulkaninfo'")?;

    if !summary_output.status.success() {
        bail!(
            "Exit code {} for 'vulkaninfo': {} {}",
            summary_output.status,
            String::from_utf8_lossy(&summary_output.stdout),
            String::from_utf8_lossy(&summary_output.stderr)
        );
    }

    let summary =
        String::from_utf8(summary_output.stdout).context("Could not parse vulkan summary")?;
    let entries = parse_summary(&summary);

    for (i, entry) in entries.into_iter().enumerate() {
        if u32::from_str_radix(entry.vendor_id, 16) == Ok(vendor_id)
            && u32::from_str_radix(entry.device_id, 16) == Ok(device_id)
        {
            let output = vulkaninfo_command()
                .arg(format!("--json={i}"))
                .current_dir("/tmp")
                .output()
                .await
                .context("Could not read vulkan info for device")?;
            if !output.status.success() {
                bail!(
                    "Exit code {} for 'vulkaninfo': {} {}",
                    output.status,
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
            }

            let info = if let Ok(devsim) =
                serde_json::from_slice::<VulkanInfoDevSimManifest>(&output.stdout)
            {
                parse_legacy_devsim(devsim, &entry)
            } else {
                let file_path = Path::new("/tmp").join(entry.file_name());
                let manifest = fs::read_to_string(&file_path).with_context(|| {
                    format!("Could not read info file from '{}'", file_path.display())
                })?;

                if let Err(err) = fs::remove_file(&file_path) {
                    error!(
                        "could not clean up file at '{}' created by vulkaninfo: {err}",
                        file_path.display()
                    );
                }

                match serde_json::from_str::<VulkanInfoManifest>(&manifest) {
                    Ok(manifest) if pci_info_matches(&manifest, &pci_slot_info) => {
                        parse_manifest(manifest, &entry)
                    }
                    Ok(_) => continue,
                    Err(err) => {
                        error!("could not parse vulkan manifest: {err}");
                        continue;
                    }
                }
            };
            results.push(info);
        }
    }

    if results.is_empty() {
        Err(anyhow!(
            "Could not find a vulkan device with matching pci ids"
        ))
    } else {
        Ok(results)
    }
}

fn pci_info_matches(manifest: &VulkanInfoManifest, slot_info: &PciSlotInfo) -> bool {
    manifest
        .capabilities
        .device
        .properties
        .pci_bus_info
        .as_ref()
        .is_none_or(|bus_info| {
            bus_info.pci_bus == slot_info.bus
                && bus_info.pci_domain == slot_info.domain
                && bus_info.pci_device == slot_info.dev
                && bus_info.pci_function == slot_info.func
        })
}

fn parse_manifest(manifest: VulkanInfoManifest, entry: &SummaryDeviceEntry<'_>) -> VulkanInfo {
    let mut extensions: IndexMap<String, bool> = manifest
        .capabilities
        .device
        .extensions
        .into_iter()
        .map(|(name, version)| (name.to_owned(), version > 0))
        .collect();

    for extension in VULKAN_EXTENSIONS {
        if let Entry::Vacant(entry) = extensions.entry(extension.to_owned()) {
            entry.insert(false);
        }
    }

    let mut features: IndexMap<String, bool> = manifest
        .capabilities
        .device
        .features
        .into_values()
        .flat_map(|fields| {
            fields
                .into_iter()
                .map(|(key, value)| (key.to_owned(), value))
        })
        .collect();

    for feature in VULKAN_FEATURES {
        if let Entry::Vacant(entry) = features.entry(feature.to_owned()) {
            entry.insert(false);
        }
    }

    VulkanInfo {
        device_name: entry.device_name.to_owned(),
        api_version: entry.api_version.to_owned(),
        driver: entry.driver_info(),
        enabled_layers: vec![],
        extensions,
        features,
    }
}

fn parse_legacy_devsim(
    devsim: VulkanInfoDevSimManifest,
    entry: &SummaryDeviceEntry<'_>,
) -> VulkanInfo {
    let mut extensions: IndexMap<String, bool> = devsim
        .extension_properties
        .into_iter()
        .map(|extension| (extension.extension_name, extension.spec_version > 0))
        .collect();

    for extension in VULKAN_EXTENSIONS {
        if let Entry::Vacant(entry) = extensions.entry(extension.to_owned()) {
            entry.insert(false);
        }
    }

    let mut features: IndexMap<String, bool> = devsim
        .features
        .into_iter()
        .map(|(name, value)| (name, value > 0))
        .collect();

    for feature in VULKAN_FEATURES {
        if let Entry::Vacant(entry) = features.entry(feature.to_owned()) {
            entry.insert(false);
        }
    }

    VulkanInfo {
        device_name: entry.device_name.to_owned(),
        api_version: entry.api_version.to_owned(),
        driver: entry.driver_info(),
        enabled_layers: vec![],
        extensions,
        features,
    }
}

#[derive(Deserialize)]
struct VulkanInfoManifest<'a> {
    #[serde(borrow)]
    capabilities: VulkanInfoCapabilities<'a>,
}

#[derive(Deserialize)]
struct VulkanInfoCapabilities<'a> {
    #[serde(borrow)]
    device: VulkanInfoCapabilitiesDevice<'a>,
}

#[derive(Deserialize)]
struct VulkanInfoCapabilitiesDevice<'a> {
    #[serde(borrow)]
    extensions: IndexMap<&'a str, i32>,
    #[serde(borrow)]
    features: IndexMap<&'a str, IndexMap<&'a str, bool>>,
    properties: VulkanInfoDeviceProperties,
}

#[derive(Deserialize)]
struct VulkanInfoDeviceProperties {
    #[serde(rename = "VkPhysicalDevicePCIBusInfoPropertiesEXT")]
    pci_bus_info: Option<VulkanPciBusInfoProperties>,
}

#[allow(clippy::struct_field_names)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct VulkanPciBusInfoProperties {
    pci_domain: u16,
    pci_bus: u16,
    pci_device: u16,
    pci_function: u16,
}

/// Old format
#[derive(Deserialize)]
struct VulkanInfoDevSimManifest {
    #[serde(rename = "ArrayOfVkExtensionProperties")]
    extension_properties: Vec<VulkanDevsimExtensionProperty>,
    #[serde(rename = "VkPhysicalDeviceFeatures")]
    features: IndexMap<String, i32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct VulkanDevsimExtensionProperty {
    extension_name: String,
    spec_version: i32,
}

#[derive(Default, Debug, PartialEq, Eq)]
struct SummaryDeviceEntry<'a> {
    pub api_version: &'a str,
    pub device_name: &'a str,
    pub driver_name: &'a str,
    pub driver_info: &'a str,
    pub driver_version: &'a str,
    pub vendor_id: &'a str,
    pub device_id: &'a str,
}

impl SummaryDeviceEntry<'_> {
    fn file_name(&self) -> String {
        format!(
            "VP_VULKANINFO_{}_{}.json",
            self.device_name.replace([' ', '.'], "_"),
            self.driver_version.replace([' ', '.'], "_")
        )
    }

    fn driver_info(&self) -> VulkanDriverInfo {
        VulkanDriverInfo {
            version: self.driver_version.parse().unwrap_or(0),
            name: Some(self.driver_name.to_owned()),
            info: Some(self.driver_info.to_owned()),
            driver_version: None,
        }
    }
}

fn parse_summary(summary: &str) -> Vec<SummaryDeviceEntry> {
    let mut lines = summary.lines();
    let mut devices = vec![];

    for line in &mut lines {
        if line == "Devices:" {
            break;
        }
    }
    // Skip separator line and gpu start
    lines.next();
    lines.next();

    let mut entry = SummaryDeviceEntry::default();

    for prop_line in lines {
        if prop_line.starts_with("GPU") {
            devices.push(entry);
            entry = SummaryDeviceEntry::default();
        }

        if let Some((key, value)) = prop_line.trim_ascii().split_once('=') {
            let value = value.trim_ascii();

            match key.trim_ascii() {
                "apiVersion" => entry.api_version = value,
                "deviceName" => entry.device_name = value,
                "driverName" => entry.driver_name = value,
                "driverInfo" => entry.driver_info = value,
                "driverVersion" => entry.driver_version = value,
                "vendorID" => entry.vendor_id = value.trim_start_matches("0x"),
                "deviceID" => entry.device_id = value.trim_start_matches("0x"),
                _ => (),
            }
        }
    }
    devices.push(entry);

    devices
}

fn vulkaninfo_command() -> Command {
    let mut cmd = if let Ok(custom_command) = env::var("VULKANINFO_COMMAND") {
        let mut split = custom_command.split_ascii_whitespace();
        let program = split
            .next()
            .expect("Could not parse provided vulkaninfo command");

        let mut cmd = Command::new(program);
        cmd.args(split);
        cmd
    } else {
        Command::new("vulkaninfo")
    };

    cmd.env("DISABLE_LAYER_AMD_SWITCHABLE_GRAPHICS_1", "1");
    cmd
}

#[cfg(test)]
mod tests {
    use super::{parse_legacy_devsim, parse_summary, VulkanInfoDevSimManifest};
    use crate::server::vulkan::SummaryDeviceEntry;
    use pretty_assertions::assert_eq;

    #[test]
    fn parse_summary_basic() {
        let summary = r#"""
==========
VULKANINFO
==========

Vulkan Instance Version: 1.3.296


Instance Extensions: count = 24
-------------------------------
VK_EXT_acquire_drm_display             : extension revision 1
VK_EXT_acquire_xlib_display            : extension revision 1
VK_EXT_debug_report                    : extension revision 10
VK_EXT_debug_utils                     : extension revision 2
VK_EXT_direct_mode_display             : extension revision 1
VK_EXT_display_surface_counter         : extension revision 1
VK_EXT_headless_surface                : extension revision 1
VK_EXT_surface_maintenance1            : extension revision 1
VK_EXT_swapchain_colorspace            : extension revision 4
VK_KHR_device_group_creation           : extension revision 1
VK_KHR_display                         : extension revision 23
VK_KHR_external_fence_capabilities     : extension revision 1
VK_KHR_external_memory_capabilities    : extension revision 1
VK_KHR_external_semaphore_capabilities : extension revision 1
VK_KHR_get_display_properties2         : extension revision 1
VK_KHR_get_physical_device_properties2 : extension revision 2
VK_KHR_get_surface_capabilities2       : extension revision 1
VK_KHR_portability_enumeration         : extension revision 1
VK_KHR_surface                         : extension revision 25
VK_KHR_surface_protected_capabilities  : extension revision 1
VK_KHR_wayland_surface                 : extension revision 6
VK_KHR_xcb_surface                     : extension revision 6
VK_KHR_xlib_surface                    : extension revision 6
VK_LUNARG_direct_driver_loading        : extension revision 1

Instance Layers: count = 13
---------------------------
VK_LAYER_FROG_gamescope_wsi_x86_64 Gamescope WSI (XWayland Bypass) Layer (x86_64) 1.3.221  version 1
VK_LAYER_MANGOAPP_overlay          Mangoapp Layer                                 1.3.0    version 1
VK_LAYER_MANGOAPP_overlay          Mangoapp Layer                                 1.3.0    version 1
VK_LAYER_MANGOHUD_overlay_x86      Vulkan Hud Overlay                             1.3.0    version 1
VK_LAYER_MANGOHUD_overlay_x86_64   Vulkan Hud Overlay                             1.3.0    version 1
VK_LAYER_MESA_device_select        Linux device selection layer                   1.3.211  version 1
VK_LAYER_NV_optimus                NVIDIA Optimus layer                           1.3.289  version 1
VK_LAYER_RENDERDOC_Capture         Debugging capture layer for RenderDoc          1.3.131  version 35
VK_LAYER_VALVE_steam_fossilize_32  Steam Pipeline Caching Layer                   1.3.207  version 1
VK_LAYER_VALVE_steam_fossilize_64  Steam Pipeline Caching Layer                   1.3.207  version 1
VK_LAYER_VALVE_steam_overlay_32    Steam Overlay Layer                            1.3.207  version 1
VK_LAYER_VALVE_steam_overlay_64    Steam Overlay Layer                            1.3.207  version 1
VK_LAYER_VKBASALT_post_processing  a post processing layer                        1.3.223  version 1

Devices:
========
GPU0:
	apiVersion         = 1.3.289
	driverVersion      = 565.77.0.0
	vendorID           = 0x10de
	deviceID           = 0x2704
	deviceType         = PHYSICAL_DEVICE_TYPE_DISCRETE_GPU
	deviceName         = NVIDIA GeForce RTX 4080
	driverID           = DRIVER_ID_NVIDIA_PROPRIETARY
	driverName         = NVIDIA
	driverInfo         = 565.77
	conformanceVersion = 1.3.8.2
	deviceUUID         = 3d28c8d2-dcc3-dcb6-1da6-5a521f0bcd6d
	driverUUID         = 5d948742-de2b-5e32-9692-c2a5621aed9a
GPU1:
	apiVersion         = 1.3.289
	driverVersion      = 0.0.1
	vendorID           = 0x10005
	deviceID           = 0x0000
	deviceType         = PHYSICAL_DEVICE_TYPE_CPU
	deviceName         = llvmpipe (LLVM 19.1.0, 256 bits)
	driverID           = DRIVER_ID_MESA_LLVMPIPE
	driverName         = llvmpipe
	driverInfo         = Mesa 24.2.8 (LLVM 19.1.0)
	conformanceVersion = 1.3.1.1
	deviceUUID         = 6d657361-3234-2e32-2e38-000000000000
	driverUUID         = 6c6c766d-7069-7065-5555-494400000000
    }
    """#;

        let expected_entries = vec![
            SummaryDeviceEntry {
                api_version: "1.3.289",
                device_name: "NVIDIA GeForce RTX 4080",
                driver_name: "NVIDIA",
                driver_version: "565.77.0.0",
                driver_info: "565.77",
                vendor_id: "10de",
                device_id: "2704",
            },
            SummaryDeviceEntry {
                api_version: "1.3.289",
                device_name: "llvmpipe (LLVM 19.1.0, 256 bits)",
                driver_name: "llvmpipe",
                driver_version: "0.0.1",
                driver_info: "Mesa 24.2.8 (LLVM 19.1.0)",
                vendor_id: "10005",
                device_id: "0000",
            },
        ];

        assert_eq!(expected_entries, parse_summary(summary));
    }

    #[allow(clippy::too_many_lines)]
    #[test]
    fn parse_legacy_output() {
        let data = r#"
{
	"$schema": "https://schema.khronos.org/vulkan/devsim_1_0_0.json#",
	"comments": {
		"desc": "JSON configuration file describing GPU 0 (llvmpipe (LLVM 15.0.7, 256 bits)). Generated using the vulkaninfo program.",
		"vulkanApiVersion": "1.3.204"
	},
	"ArrayOfVkLayerProperties": [
		{
			"layerName": "VK_LAYER_INTEL_nullhw",
			"specVersion": 4198473,
			"implementationVersion": 1,
			"description": "INTEL NULL HW"
		},
		{
			"layerName": "VK_LAYER_MESA_device_select",
			"specVersion": 4206803,
			"implementationVersion": 1,
			"description": "Linux device selection layer"
		},
		{
			"layerName": "VK_LAYER_MESA_overlay",
			"specVersion": 4206803,
			"implementationVersion": 1,
			"description": "Mesa Overlay layer"
		}
	],
	"VkPhysicalDeviceProperties": {
		"apiVersion": 4206847,
		"driverVersion": 1,
		"vendorID": 65541,
		"deviceID": 0,
		"deviceType": 4,
		"deviceName": "llvmpipe (LLVM 15.0.7, 256 bits)",
		"pipelineCacheUUID": [
			50,
			51,
			46,
			50,
			46,
			49,
			45,
			49,
			117,
			98,
			117,
			110,
			116,
			117,
			51,
			46
		],
		"limits": {
			"maxImageDimension1D": 16384,
			"maxImageDimension2D": 16384,
			"maxImageDimension3D": 4096,
			"maxImageDimensionCube": 32768,
			"maxImageArrayLayers": 2048,
			"maxTexelBufferElements": 134217728,
			"maxUniformBufferRange": 65536,
			"maxStorageBufferRange": 134217728,
			"maxPushConstantsSize": 256,
			"maxMemoryAllocationCount": 4294967295,
			"maxSamplerAllocationCount": 32768,
			"bufferImageGranularity": 64,
			"sparseAddressSpaceSize": 0,
			"maxBoundDescriptorSets": 8,
			"maxPerStageDescriptorSamplers": 1000000,
			"maxPerStageDescriptorUniformBuffers": 1000000,
			"maxPerStageDescriptorStorageBuffers": 1000000,
			"maxPerStageDescriptorSampledImages": 1000000,
			"maxPerStageDescriptorStorageImages": 1000000,
			"maxPerStageDescriptorInputAttachments": 1000000,
			"maxPerStageResources": 1000000,
			"maxDescriptorSetSamplers": 1000000,
			"maxDescriptorSetUniformBuffers": 1000000,
			"maxDescriptorSetUniformBuffersDynamic": 1000000,
			"maxDescriptorSetStorageBuffers": 1000000,
			"maxDescriptorSetStorageBuffersDynamic": 1000000,
			"maxDescriptorSetSampledImages": 1000000,
			"maxDescriptorSetStorageImages": 1000000,
			"maxDescriptorSetInputAttachments": 1000000,
			"maxVertexInputAttributes": 32,
			"maxVertexInputBindings": 32,
			"maxVertexInputAttributeOffset": 2047,
			"maxVertexInputBindingStride": 2048,
			"maxVertexOutputComponents": 128,
			"maxTessellationGenerationLevel": 64,
			"maxTessellationPatchSize": 32,
			"maxTessellationControlPerVertexInputComponents": 128,
			"maxTessellationControlPerVertexOutputComponents": 128,
			"maxTessellationControlPerPatchOutputComponents": 128,
			"maxTessellationControlTotalOutputComponents": 4096,
			"maxTessellationEvaluationInputComponents": 128,
			"maxTessellationEvaluationOutputComponents": 128,
			"maxGeometryShaderInvocations": 32,
			"maxGeometryInputComponents": 64,
			"maxGeometryOutputComponents": 128,
			"maxGeometryOutputVertices": 1024,
			"maxGeometryTotalOutputComponents": 1024,
			"maxFragmentInputComponents": 128,
			"maxFragmentOutputAttachments": 8,
			"maxFragmentDualSrcAttachments": 2,
			"maxFragmentCombinedOutputResources": 104,
			"maxComputeSharedMemorySize": 32768,
			"maxComputeWorkGroupCount": [
				65535,
				65535,
				65535
			],
			"maxComputeWorkGroupInvocations": 1024,
			"maxComputeWorkGroupSize": [
				1024,
				1024,
				1024
			],
			"subPixelPrecisionBits": 8,
			"subTexelPrecisionBits": 8,
			"mipmapPrecisionBits": 4,
			"maxDrawIndexedIndexValue": 4294967295,
			"maxDrawIndirectCount": 4294967295,
			"maxSamplerLodBias": 16,
			"maxSamplerAnisotropy": 16,
			"maxViewports": 16,
			"maxViewportDimensions": [
				16384,
				16384
			],
			"viewportBoundsRange": [
				-32768,
				32768
			],
			"viewportSubPixelBits": 0,
			"minMemoryMapAlignment": 64,
			"minTexelBufferOffsetAlignment": 16,
			"minUniformBufferOffsetAlignment": 16,
			"minStorageBufferOffsetAlignment": 16,
			"minTexelOffset": -32,
			"maxTexelOffset": 31,
			"minTexelGatherOffset": -32,
			"maxTexelGatherOffset": 31,
			"minInterpolationOffset": -2,
			"maxInterpolationOffset": 2,
			"subPixelInterpolationOffsetBits": 8,
			"maxFramebufferWidth": 16384,
			"maxFramebufferHeight": 16384,
			"maxFramebufferLayers": 2048,
			"framebufferColorSampleCounts": 5,
			"framebufferDepthSampleCounts": 5,
			"framebufferStencilSampleCounts": 5,
			"framebufferNoAttachmentsSampleCounts": 5,
			"maxColorAttachments": 8,
			"sampledImageColorSampleCounts": 5,
			"sampledImageIntegerSampleCounts": 5,
			"sampledImageDepthSampleCounts": 5,
			"sampledImageStencilSampleCounts": 5,
			"storageImageSampleCounts": 5,
			"maxSampleMaskWords": 1,
			"timestampComputeAndGraphics": 1,
			"timestampPeriod": 1,
			"maxClipDistances": 8,
			"maxCullDistances": 8,
			"maxCombinedClipAndCullDistances": 8,
			"discreteQueuePriorities": 2,
			"pointSizeRange": [
				0,
				255
			],
			"lineWidthRange": [
				1,
				255
			],
			"pointSizeGranularity": 0.125,
			"lineWidthGranularity": 0.0078125,
			"strictLines": 1,
			"standardSampleLocations": 1,
			"optimalBufferCopyOffsetAlignment": 128,
			"optimalBufferCopyRowPitchAlignment": 128,
			"nonCoherentAtomSize": 64
		},
		"sparseProperties": {
			"residencyStandard2DBlockShape": 0,
			"residencyStandard2DMultisampleBlockShape": 0,
			"residencyStandard3DBlockShape": 0,
			"residencyAlignedMipSize": 0,
			"residencyNonResidentStrict": 0
		}
	},
	"ArrayOfVkQueueFamilyProperties": [
		{
			"minImageTransferGranularity": {
				"width": 1,
				"height": 1,
				"depth": 1
			},
			"queueCount": 1,
			"queueFlags": 7,
			"timestampValidBits": 64
		}
	],
	"ArrayOfVkExtensionProperties": [
		{
			"extensionName": "VK_KHR_8bit_storage",
			"specVersion": 1
		},
		{
			"extensionName": "VK_KHR_16bit_storage",
			"specVersion": 1
		},
		{
			"extensionName": "VK_KHR_bind_memory2",
			"specVersion": 1
		},
		{
			"extensionName": "VK_KHR_buffer_device_address",
			"specVersion": 1
		},
		{
			"extensionName": "VK_KHR_copy_commands2",
			"specVersion": 1
		},
		{
			"extensionName": "VK_KHR_create_renderpass2",
			"specVersion": 1
		},
		{
			"extensionName": "VK_KHR_dedicated_allocation",
			"specVersion": 3
		},
		{
			"extensionName": "VK_KHR_depth_stencil_resolve",
			"specVersion": 1
		},
		{
			"extensionName": "VK_KHR_descriptor_update_template",
			"specVersion": 1
		},
		{
			"extensionName": "VK_KHR_device_group",
			"specVersion": 4
		},
		{
			"extensionName": "VK_KHR_draw_indirect_count",
			"specVersion": 1
		},
		{
			"extensionName": "VK_KHR_driver_properties",
			"specVersion": 1
		},
		{
			"extensionName": "VK_KHR_dynamic_rendering",
			"specVersion": 1
		},
		{
			"extensionName": "VK_KHR_external_fence",
			"specVersion": 1
		},
		{
			"extensionName": "VK_KHR_external_memory",
			"specVersion": 1
		},
		{
			"extensionName": "VK_KHR_external_memory_fd",
			"specVersion": 1
		},
		{
			"extensionName": "VK_KHR_external_semaphore",
			"specVersion": 1
		},
		{
			"extensionName": "VK_KHR_format_feature_flags2",
			"specVersion": 2
		},
		{
			"extensionName": "VK_KHR_get_memory_requirements2",
			"specVersion": 1
		},
		{
			"extensionName": "VK_KHR_image_format_list",
			"specVersion": 1
		},
		{
			"extensionName": "VK_KHR_imageless_framebuffer",
			"specVersion": 1
		},
		{
			"extensionName": "VK_KHR_incremental_present",
			"specVersion": 2
		},
		{
			"extensionName": "VK_KHR_maintenance1",
			"specVersion": 2
		},
		{
			"extensionName": "VK_KHR_maintenance2",
			"specVersion": 1
		},
		{
			"extensionName": "VK_KHR_maintenance3",
			"specVersion": 1
		},
		{
			"extensionName": "VK_KHR_maintenance4",
			"specVersion": 2
		},
		{
			"extensionName": "VK_KHR_multiview",
			"specVersion": 1
		},
		{
			"extensionName": "VK_KHR_pipeline_library",
			"specVersion": 1
		},
		{
			"extensionName": "VK_KHR_push_descriptor",
			"specVersion": 2
		},
		{
			"extensionName": "VK_KHR_relaxed_block_layout",
			"specVersion": 1
		},
		{
			"extensionName": "VK_KHR_sampler_mirror_clamp_to_edge",
			"specVersion": 3
		},
		{
			"extensionName": "VK_KHR_separate_depth_stencil_layouts",
			"specVersion": 1
		},
		{
			"extensionName": "VK_KHR_shader_atomic_int64",
			"specVersion": 1
		},
		{
			"extensionName": "VK_KHR_shader_clock",
			"specVersion": 1
		},
		{
			"extensionName": "VK_KHR_shader_draw_parameters",
			"specVersion": 1
		},
		{
			"extensionName": "VK_KHR_shader_float16_int8",
			"specVersion": 1
		},
		{
			"extensionName": "VK_KHR_shader_float_controls",
			"specVersion": 4
		},
		{
			"extensionName": "VK_KHR_shader_integer_dot_product",
			"specVersion": 1
		},
		{
			"extensionName": "VK_KHR_shader_non_semantic_info",
			"specVersion": 1
		},
		{
			"extensionName": "VK_KHR_shader_subgroup_extended_types",
			"specVersion": 1
		},
		{
			"extensionName": "VK_KHR_shader_terminate_invocation",
			"specVersion": 1
		},
		{
			"extensionName": "VK_KHR_spirv_1_4",
			"specVersion": 1
		},
		{
			"extensionName": "VK_KHR_storage_buffer_storage_class",
			"specVersion": 1
		},
		{
			"extensionName": "VK_KHR_swapchain",
			"specVersion": 70
		},
		{
			"extensionName": "VK_KHR_swapchain_mutable_format",
			"specVersion": 1
		},
		{
			"extensionName": "VK_KHR_synchronization2",
			"specVersion": 1
		},
		{
			"extensionName": "VK_KHR_timeline_semaphore",
			"specVersion": 2
		},
		{
			"extensionName": "VK_KHR_uniform_buffer_standard_layout",
			"specVersion": 1
		},
		{
			"extensionName": "VK_KHR_variable_pointers",
			"specVersion": 1
		},
		{
			"extensionName": "VK_KHR_vulkan_memory_model",
			"specVersion": 3
		},
		{
			"extensionName": "VK_KHR_zero_initialize_workgroup_memory",
			"specVersion": 1
		},
		{
			"extensionName": "VK_EXT_4444_formats",
			"specVersion": 1
		},
		{
			"extensionName": "VK_EXT_attachment_feedback_loop_dynamic_state",
			"specVersion": 1
		},
		{
			"extensionName": "VK_EXT_attachment_feedback_loop_layout",
			"specVersion": 2
		},
		{
			"extensionName": "VK_EXT_border_color_swizzle",
			"specVersion": 1
		},
		{
			"extensionName": "VK_EXT_calibrated_timestamps",
			"specVersion": 2
		},
		{
			"extensionName": "VK_EXT_color_write_enable",
			"specVersion": 1
		},
		{
			"extensionName": "VK_EXT_conditional_rendering",
			"specVersion": 2
		},
		{
			"extensionName": "VK_EXT_custom_border_color",
			"specVersion": 12
		},
		{
			"extensionName": "VK_EXT_depth_clip_control",
			"specVersion": 1
		},
		{
			"extensionName": "VK_EXT_depth_clip_enable",
			"specVersion": 1
		},
		{
			"extensionName": "VK_EXT_depth_range_unrestricted",
			"specVersion": 1
		},
		{
			"extensionName": "VK_EXT_descriptor_buffer",
			"specVersion": 1
		},
		{
			"extensionName": "VK_EXT_descriptor_indexing",
			"specVersion": 2
		},
		{
			"extensionName": "VK_EXT_dynamic_rendering_unused_attachments",
			"specVersion": 1
		},
		{
			"extensionName": "VK_EXT_extended_dynamic_state",
			"specVersion": 1
		},
		{
			"extensionName": "VK_EXT_extended_dynamic_state2",
			"specVersion": 1
		},
		{
			"extensionName": "VK_EXT_extended_dynamic_state3",
			"specVersion": 2
		},
		{
			"extensionName": "VK_EXT_external_memory_host",
			"specVersion": 1
		},
		{
			"extensionName": "VK_EXT_graphics_pipeline_library",
			"specVersion": 1
		},
		{
			"extensionName": "VK_EXT_host_query_reset",
			"specVersion": 1
		},
		{
			"extensionName": "VK_EXT_image_2d_view_of_3d",
			"specVersion": 1
		},
		{
			"extensionName": "VK_EXT_image_robustness",
			"specVersion": 1
		},
		{
			"extensionName": "VK_EXT_image_sliced_view_of_3d",
			"specVersion": 1
		},
		{
			"extensionName": "VK_EXT_index_type_uint8",
			"specVersion": 1
		},
		{
			"extensionName": "VK_EXT_inline_uniform_block",
			"specVersion": 1
		},
		{
			"extensionName": "VK_EXT_line_rasterization",
			"specVersion": 1
		},
		{
			"extensionName": "VK_EXT_memory_budget",
			"specVersion": 1
		},
		{
			"extensionName": "VK_EXT_memory_priority",
			"specVersion": 1
		},
		{
			"extensionName": "VK_EXT_mesh_shader",
			"specVersion": 1
		},
		{
			"extensionName": "VK_EXT_multi_draw",
			"specVersion": 1
		},
		{
			"extensionName": "VK_EXT_multisampled_render_to_single_sampled",
			"specVersion": 1
		},
		{
			"extensionName": "VK_EXT_mutable_descriptor_type",
			"specVersion": 1
		},
		{
			"extensionName": "VK_EXT_non_seamless_cube_map",
			"specVersion": 1
		},
		{
			"extensionName": "VK_EXT_pageable_device_local_memory",
			"specVersion": 1
		},
		{
			"extensionName": "VK_EXT_pipeline_creation_cache_control",
			"specVersion": 3
		},
		{
			"extensionName": "VK_EXT_pipeline_creation_feedback",
			"specVersion": 1
		},
		{
			"extensionName": "VK_EXT_post_depth_coverage",
			"specVersion": 1
		},
		{
			"extensionName": "VK_EXT_primitive_topology_list_restart",
			"specVersion": 1
		},
		{
			"extensionName": "VK_EXT_primitives_generated_query",
			"specVersion": 1
		},
		{
			"extensionName": "VK_EXT_private_data",
			"specVersion": 1
		},
		{
			"extensionName": "VK_EXT_provoking_vertex",
			"specVersion": 1
		},
		{
			"extensionName": "VK_EXT_rasterization_order_attachment_access",
			"specVersion": 1
		},
		{
			"extensionName": "VK_EXT_robustness2",
			"specVersion": 1
		},
		{
			"extensionName": "VK_EXT_sampler_filter_minmax",
			"specVersion": 2
		},
		{
			"extensionName": "VK_EXT_scalar_block_layout",
			"specVersion": 1
		},
		{
			"extensionName": "VK_EXT_separate_stencil_usage",
			"specVersion": 1
		},
		{
			"extensionName": "VK_EXT_shader_atomic_float",
			"specVersion": 1
		},
		{
			"extensionName": "VK_EXT_shader_atomic_float2",
			"specVersion": 1
		},
		{
			"extensionName": "VK_EXT_shader_demote_to_helper_invocation",
			"specVersion": 1
		},
		{
			"extensionName": "VK_EXT_shader_object",
			"specVersion": 1
		},
		{
			"extensionName": "VK_EXT_shader_stencil_export",
			"specVersion": 1
		},
		{
			"extensionName": "VK_EXT_shader_subgroup_ballot",
			"specVersion": 1
		},
		{
			"extensionName": "VK_EXT_shader_subgroup_vote",
			"specVersion": 1
		},
		{
			"extensionName": "VK_EXT_shader_viewport_index_layer",
			"specVersion": 1
		},
		{
			"extensionName": "VK_EXT_subgroup_size_control",
			"specVersion": 2
		},
		{
			"extensionName": "VK_EXT_texel_buffer_alignment",
			"specVersion": 1
		},
		{
			"extensionName": "VK_EXT_transform_feedback",
			"specVersion": 1
		},
		{
			"extensionName": "VK_EXT_vertex_attribute_divisor",
			"specVersion": 3
		},
		{
			"extensionName": "VK_EXT_vertex_input_dynamic_state",
			"specVersion": 2
		},
		{
			"extensionName": "VK_ARM_rasterization_order_attachment_access",
			"specVersion": 1
		},
		{
			"extensionName": "VK_GOOGLE_decorate_string",
			"specVersion": 1
		},
		{
			"extensionName": "VK_GOOGLE_hlsl_functionality1",
			"specVersion": 1
		},
		{
			"extensionName": "VK_NV_device_generated_commands",
			"specVersion": 3
		}
	],
	"VkPhysicalDeviceMemoryProperties": {
		"memoryHeaps": [
			{
				"flags": 1,
				"size": 33544609792
			}
		],
		"memoryTypes": [
			{
				"heapIndex": 0,
				"propertyFlags": 15
			}
		]
	},
	"VkPhysicalDeviceFeatures": {
		"robustBufferAccess": 1,
		"fullDrawIndexUint32": 1,
		"imageCubeArray": 1,
		"independentBlend": 1,
		"geometryShader": 1,
		"tessellationShader": 1,
		"sampleRateShading": 1,
		"dualSrcBlend": 1,
		"logicOp": 1,
		"multiDrawIndirect": 1,
		"drawIndirectFirstInstance": 1,
		"depthClamp": 1,
		"depthBiasClamp": 1,
		"fillModeNonSolid": 1,
		"depthBounds": 0,
		"wideLines": 1,
		"largePoints": 1,
		"alphaToOne": 1,
		"multiViewport": 1,
		"samplerAnisotropy": 1,
		"textureCompressionETC2": 0,
		"textureCompressionASTC_LDR": 0,
		"textureCompressionBC": 1,
		"occlusionQueryPrecise": 1,
		"pipelineStatisticsQuery": 1,
		"vertexPipelineStoresAndAtomics": 1,
		"fragmentStoresAndAtomics": 1,
		"shaderTessellationAndGeometryPointSize": 1,
		"shaderImageGatherExtended": 1,
		"shaderStorageImageExtendedFormats": 1,
		"shaderStorageImageMultisample": 1,
		"shaderStorageImageReadWithoutFormat": 1,
		"shaderStorageImageWriteWithoutFormat": 1,
		"shaderUniformBufferArrayDynamicIndexing": 1,
		"shaderSampledImageArrayDynamicIndexing": 1,
		"shaderStorageBufferArrayDynamicIndexing": 1,
		"shaderStorageImageArrayDynamicIndexing": 1,
		"shaderClipDistance": 1,
		"shaderCullDistance": 1,
		"shaderFloat64": 1,
		"shaderInt64": 1,
		"shaderInt16": 1,
		"shaderResourceResidency": 0,
		"shaderResourceMinLod": 0,
		"sparseBinding": 0,
		"sparseResidencyBuffer": 0,
		"sparseResidencyImage2D": 0,
		"sparseResidencyImage3D": 0,
		"sparseResidency2Samples": 0,
		"sparseResidency4Samples": 0,
		"sparseResidency8Samples": 0,
		"sparseResidency16Samples": 0,
		"sparseResidencyAliased": 0,
		"variableMultisampleRate": 0,
		"inheritedQueries": 0
	},
	"ArrayOfVkFormatProperties": [
		{
			"formatID": 3,
			"linearTilingFeatures": 56705,
			"optimalTilingFeatures": 56705,
			"bufferFeatures": 72
		},
		{
			"formatID": 4,
			"linearTilingFeatures": 56705,
			"optimalTilingFeatures": 56705,
			"bufferFeatures": 72
		},
		{
			"formatID": 5,
			"linearTilingFeatures": 53633,
			"optimalTilingFeatures": 53633,
			"bufferFeatures": 72
		},
		{
			"formatID": 6,
			"linearTilingFeatures": 56705,
			"optimalTilingFeatures": 56705,
			"bufferFeatures": 72
		},
		{
			"formatID": 7,
			"linearTilingFeatures": 56705,
			"optimalTilingFeatures": 56705,
			"bufferFeatures": 72
		},
		{
			"formatID": 8,
			"linearTilingFeatures": 56705,
			"optimalTilingFeatures": 56705,
			"bufferFeatures": 72
		},
		{
			"formatID": 9,
			"linearTilingFeatures": 122243,
			"optimalTilingFeatures": 122243,
			"bufferFeatures": 88
		},
		{
			"formatID": 10,
			"linearTilingFeatures": 121987,
			"optimalTilingFeatures": 121987,
			"bufferFeatures": 88
		},
		{
			"formatID": 11,
			"linearTilingFeatures": 3072,
			"optimalTilingFeatures": 3072,
			"bufferFeatures": 64
		},
		{
			"formatID": 12,
			"linearTilingFeatures": 3072,
			"optimalTilingFeatures": 3072,
			"bufferFeatures": 64
		},
		{
			"formatID": 13,
			"linearTilingFeatures": 52355,
			"optimalTilingFeatures": 52355,
			"bufferFeatures": 88
		},
		{
			"formatID": 14,
			"linearTilingFeatures": 52355,
			"optimalTilingFeatures": 52355,
			"bufferFeatures": 88
		},
		{
			"formatID": 16,
			"linearTilingFeatures": 56707,
			"optimalTilingFeatures": 56707,
			"bufferFeatures": 88
		},
		{
			"formatID": 17,
			"linearTilingFeatures": 56451,
			"optimalTilingFeatures": 56451,
			"bufferFeatures": 88
		},
		{
			"formatID": 18,
			"linearTilingFeatures": 3072,
			"optimalTilingFeatures": 3072,
			"bufferFeatures": 64
		},
		{
			"formatID": 19,
			"linearTilingFeatures": 3072,
			"optimalTilingFeatures": 3072,
			"bufferFeatures": 64
		},
		{
			"formatID": 20,
			"linearTilingFeatures": 52355,
			"optimalTilingFeatures": 52355,
			"bufferFeatures": 88
		},
		{
			"formatID": 21,
			"linearTilingFeatures": 52355,
			"optimalTilingFeatures": 52355,
			"bufferFeatures": 88
		},
		{
			"formatID": 23,
			"linearTilingFeatures": 0,
			"optimalTilingFeatures": 0,
			"bufferFeatures": 72
		},
		{
			"formatID": 24,
			"linearTilingFeatures": 0,
			"optimalTilingFeatures": 0,
			"bufferFeatures": 72
		},
		{
			"formatID": 25,
			"linearTilingFeatures": 0,
			"optimalTilingFeatures": 0,
			"bufferFeatures": 64
		},
		{
			"formatID": 26,
			"linearTilingFeatures": 0,
			"optimalTilingFeatures": 0,
			"bufferFeatures": 64
		},
		{
			"formatID": 27,
			"linearTilingFeatures": 0,
			"optimalTilingFeatures": 0,
			"bufferFeatures": 72
		},
		{
			"formatID": 28,
			"linearTilingFeatures": 0,
			"optimalTilingFeatures": 0,
			"bufferFeatures": 72
		},
		{
			"formatID": 29,
			"linearTilingFeatures": 0,
			"optimalTilingFeatures": 0,
			"bufferFeatures": 8
		},
		{
			"formatID": 30,
			"linearTilingFeatures": 0,
			"optimalTilingFeatures": 0,
			"bufferFeatures": 72
		},
		{
			"formatID": 31,
			"linearTilingFeatures": 0,
			"optimalTilingFeatures": 0,
			"bufferFeatures": 72
		},
		{
			"formatID": 32,
			"linearTilingFeatures": 0,
			"optimalTilingFeatures": 0,
			"bufferFeatures": 64
		},
		{
			"formatID": 33,
			"linearTilingFeatures": 0,
			"optimalTilingFeatures": 0,
			"bufferFeatures": 64
		},
		{
			"formatID": 34,
			"linearTilingFeatures": 0,
			"optimalTilingFeatures": 0,
			"bufferFeatures": 72
		},
		{
			"formatID": 35,
			"linearTilingFeatures": 0,
			"optimalTilingFeatures": 0,
			"bufferFeatures": 72
		},
		{
			"formatID": 36,
			"linearTilingFeatures": 0,
			"optimalTilingFeatures": 0,
			"bufferFeatures": 8
		},
		{
			"formatID": 37,
			"linearTilingFeatures": 56707,
			"optimalTilingFeatures": 56707,
			"bufferFeatures": 88
		},
		{
			"formatID": 38,
			"linearTilingFeatures": 56451,
			"optimalTilingFeatures": 56451,
			"bufferFeatures": 88
		},
		{
			"formatID": 39,
			"linearTilingFeatures": 3072,
			"optimalTilingFeatures": 3072,
			"bufferFeatures": 64
		},
		{
			"formatID": 40,
			"linearTilingFeatures": 3072,
			"optimalTilingFeatures": 3072,
			"bufferFeatures": 64
		},
		{
			"formatID": 41,
			"linearTilingFeatures": 52355,
			"optimalTilingFeatures": 52355,
			"bufferFeatures": 88
		},
		{
			"formatID": 42,
			"linearTilingFeatures": 52355,
			"optimalTilingFeatures": 52355,
			"bufferFeatures": 88
		},
		{
			"formatID": 43,
			"linearTilingFeatures": 56705,
			"optimalTilingFeatures": 56705,
			"bufferFeatures": 8
		},
		{
			"formatID": 44,
			"linearTilingFeatures": 56707,
			"optimalTilingFeatures": 56707,
			"bufferFeatures": 88
		},
		{
			"formatID": 45,
			"linearTilingFeatures": 56449,
			"optimalTilingFeatures": 56449,
			"bufferFeatures": 72
		},
		{
			"formatID": 46,
			"linearTilingFeatures": 3072,
			"optimalTilingFeatures": 3072,
			"bufferFeatures": 64
		},
		{
			"formatID": 47,
			"linearTilingFeatures": 3072,
			"optimalTilingFeatures": 3072,
			"bufferFeatures": 64
		},
		{
			"formatID": 48,
			"linearTilingFeatures": 52353,
			"optimalTilingFeatures": 52353,
			"bufferFeatures": 72
		},
		{
			"formatID": 49,
			"linearTilingFeatures": 52353,
			"optimalTilingFeatures": 52353,
			"bufferFeatures": 72
		},
		{
			"formatID": 50,
			"linearTilingFeatures": 56705,
			"optimalTilingFeatures": 56705,
			"bufferFeatures": 8
		},
		{
			"formatID": 51,
			"linearTilingFeatures": 56707,
			"optimalTilingFeatures": 56707,
			"bufferFeatures": 88
		},
		{
			"formatID": 52,
			"linearTilingFeatures": 56451,
			"optimalTilingFeatures": 56451,
			"bufferFeatures": 88
		},
		{
			"formatID": 53,
			"linearTilingFeatures": 3072,
			"optimalTilingFeatures": 3072,
			"bufferFeatures": 64
		},
		{
			"formatID": 54,
			"linearTilingFeatures": 3072,
			"optimalTilingFeatures": 3072,
			"bufferFeatures": 64
		},
		{
			"formatID": 55,
			"linearTilingFeatures": 52355,
			"optimalTilingFeatures": 52355,
			"bufferFeatures": 88
		},
		{
			"formatID": 56,
			"linearTilingFeatures": 52355,
			"optimalTilingFeatures": 52355,
			"bufferFeatures": 88
		},
		{
			"formatID": 57,
			"linearTilingFeatures": 56705,
			"optimalTilingFeatures": 56705,
			"bufferFeatures": 8
		},
		{
			"formatID": 58,
			"linearTilingFeatures": 53633,
			"optimalTilingFeatures": 53633,
			"bufferFeatures": 72
		},
		{
			"formatID": 59,
			"linearTilingFeatures": 53377,
			"optimalTilingFeatures": 53377,
			"bufferFeatures": 72
		},
		{
			"formatID": 60,
			"linearTilingFeatures": 3072,
			"optimalTilingFeatures": 3072,
			"bufferFeatures": 64
		},
		{
			"formatID": 61,
			"linearTilingFeatures": 3072,
			"optimalTilingFeatures": 3072,
			"bufferFeatures": 64
		},
		{
			"formatID": 62,
			"linearTilingFeatures": 52353,
			"optimalTilingFeatures": 52353,
			"bufferFeatures": 72
		},
		{
			"formatID": 64,
			"linearTilingFeatures": 56707,
			"optimalTilingFeatures": 56707,
			"bufferFeatures": 88
		},
		{
			"formatID": 65,
			"linearTilingFeatures": 53377,
			"optimalTilingFeatures": 53377,
			"bufferFeatures": 72
		},
		{
			"formatID": 66,
			"linearTilingFeatures": 3072,
			"optimalTilingFeatures": 3072,
			"bufferFeatures": 64
		},
		{
			"formatID": 67,
			"linearTilingFeatures": 3072,
			"optimalTilingFeatures": 3072,
			"bufferFeatures": 64
		},
		{
			"formatID": 68,
			"linearTilingFeatures": 52355,
			"optimalTilingFeatures": 52355,
			"bufferFeatures": 88
		},
		{
			"formatID": 70,
			"linearTilingFeatures": 122243,
			"optimalTilingFeatures": 122243,
			"bufferFeatures": 88
		},
		{
			"formatID": 71,
			"linearTilingFeatures": 121987,
			"optimalTilingFeatures": 121987,
			"bufferFeatures": 88
		},
		{
			"formatID": 72,
			"linearTilingFeatures": 3072,
			"optimalTilingFeatures": 3072,
			"bufferFeatures": 64
		},
		{
			"formatID": 73,
			"linearTilingFeatures": 3072,
			"optimalTilingFeatures": 3072,
			"bufferFeatures": 64
		},
		{
			"formatID": 74,
			"linearTilingFeatures": 52355,
			"optimalTilingFeatures": 52355,
			"bufferFeatures": 88
		},
		{
			"formatID": 75,
			"linearTilingFeatures": 52355,
			"optimalTilingFeatures": 52355,
			"bufferFeatures": 88
		},
		{
			"formatID": 76,
			"linearTilingFeatures": 122243,
			"optimalTilingFeatures": 122243,
			"bufferFeatures": 88
		},
		{
			"formatID": 77,
			"linearTilingFeatures": 56707,
			"optimalTilingFeatures": 56707,
			"bufferFeatures": 88
		},
		{
			"formatID": 78,
			"linearTilingFeatures": 56451,
			"optimalTilingFeatures": 56451,
			"bufferFeatures": 88
		},
		{
			"formatID": 79,
			"linearTilingFeatures": 3072,
			"optimalTilingFeatures": 3072,
			"bufferFeatures": 64
		},
		{
			"formatID": 80,
			"linearTilingFeatures": 3072,
			"optimalTilingFeatures": 3072,
			"bufferFeatures": 64
		},
		{
			"formatID": 81,
			"linearTilingFeatures": 52355,
			"optimalTilingFeatures": 52355,
			"bufferFeatures": 88
		},
		{
			"formatID": 82,
			"linearTilingFeatures": 52355,
			"optimalTilingFeatures": 52355,
			"bufferFeatures": 88
		},
		{
			"formatID": 83,
			"linearTilingFeatures": 56707,
			"optimalTilingFeatures": 56707,
			"bufferFeatures": 88
		},
		{
			"formatID": 84,
			"linearTilingFeatures": 0,
			"optimalTilingFeatures": 0,
			"bufferFeatures": 72
		},
		{
			"formatID": 85,
			"linearTilingFeatures": 0,
			"optimalTilingFeatures": 0,
			"bufferFeatures": 72
		},
		{
			"formatID": 86,
			"linearTilingFeatures": 0,
			"optimalTilingFeatures": 0,
			"bufferFeatures": 64
		},
		{
			"formatID": 87,
			"linearTilingFeatures": 0,
			"optimalTilingFeatures": 0,
			"bufferFeatures": 64
		},
		{
			"formatID": 88,
			"linearTilingFeatures": 0,
			"optimalTilingFeatures": 0,
			"bufferFeatures": 72
		},
		{
			"formatID": 89,
			"linearTilingFeatures": 0,
			"optimalTilingFeatures": 0,
			"bufferFeatures": 72
		},
		{
			"formatID": 90,
			"linearTilingFeatures": 0,
			"optimalTilingFeatures": 0,
			"bufferFeatures": 72
		},
		{
			"formatID": 91,
			"linearTilingFeatures": 56707,
			"optimalTilingFeatures": 56707,
			"bufferFeatures": 88
		},
		{
			"formatID": 92,
			"linearTilingFeatures": 56451,
			"optimalTilingFeatures": 56451,
			"bufferFeatures": 88
		},
		{
			"formatID": 93,
			"linearTilingFeatures": 3072,
			"optimalTilingFeatures": 3072,
			"bufferFeatures": 64
		},
		{
			"formatID": 94,
			"linearTilingFeatures": 3072,
			"optimalTilingFeatures": 3072,
			"bufferFeatures": 64
		},
		{
			"formatID": 95,
			"linearTilingFeatures": 52355,
			"optimalTilingFeatures": 52355,
			"bufferFeatures": 88
		},
		{
			"formatID": 96,
			"linearTilingFeatures": 52355,
			"optimalTilingFeatures": 52355,
			"bufferFeatures": 88
		},
		{
			"formatID": 97,
			"linearTilingFeatures": 56707,
			"optimalTilingFeatures": 56707,
			"bufferFeatures": 88
		},
		{
			"formatID": 98,
			"linearTilingFeatures": 52359,
			"optimalTilingFeatures": 52359,
			"bufferFeatures": 120
		},
		{
			"formatID": 99,
			"linearTilingFeatures": 52359,
			"optimalTilingFeatures": 52359,
			"bufferFeatures": 120
		},
		{
			"formatID": 100,
			"linearTilingFeatures": 122247,
			"optimalTilingFeatures": 122247,
			"bufferFeatures": 120
		},
		{
			"formatID": 101,
			"linearTilingFeatures": 52355,
			"optimalTilingFeatures": 52355,
			"bufferFeatures": 88
		},
		{
			"formatID": 102,
			"linearTilingFeatures": 52355,
			"optimalTilingFeatures": 52355,
			"bufferFeatures": 88
		},
		{
			"formatID": 103,
			"linearTilingFeatures": 56707,
			"optimalTilingFeatures": 56707,
			"bufferFeatures": 88
		},
		{
			"formatID": 104,
			"linearTilingFeatures": 49281,
			"optimalTilingFeatures": 49281,
			"bufferFeatures": 72
		},
		{
			"formatID": 105,
			"linearTilingFeatures": 49281,
			"optimalTilingFeatures": 49281,
			"bufferFeatures": 72
		},
		{
			"formatID": 106,
			"linearTilingFeatures": 53633,
			"optimalTilingFeatures": 53633,
			"bufferFeatures": 72
		},
		{
			"formatID": 107,
			"linearTilingFeatures": 52355,
			"optimalTilingFeatures": 52355,
			"bufferFeatures": 88
		},
		{
			"formatID": 108,
			"linearTilingFeatures": 52355,
			"optimalTilingFeatures": 52355,
			"bufferFeatures": 88
		},
		{
			"formatID": 109,
			"linearTilingFeatures": 56707,
			"optimalTilingFeatures": 56707,
			"bufferFeatures": 88
		},
		{
			"formatID": 110,
			"linearTilingFeatures": 3072,
			"optimalTilingFeatures": 3072,
			"bufferFeatures": 72
		},
		{
			"formatID": 111,
			"linearTilingFeatures": 3072,
			"optimalTilingFeatures": 3072,
			"bufferFeatures": 72
		},
		{
			"formatID": 113,
			"linearTilingFeatures": 3072,
			"optimalTilingFeatures": 3072,
			"bufferFeatures": 72
		},
		{
			"formatID": 114,
			"linearTilingFeatures": 3072,
			"optimalTilingFeatures": 3072,
			"bufferFeatures": 72
		},
		{
			"formatID": 116,
			"linearTilingFeatures": 0,
			"optimalTilingFeatures": 0,
			"bufferFeatures": 72
		},
		{
			"formatID": 117,
			"linearTilingFeatures": 0,
			"optimalTilingFeatures": 0,
			"bufferFeatures": 72
		},
		{
			"formatID": 119,
			"linearTilingFeatures": 3072,
			"optimalTilingFeatures": 3072,
			"bufferFeatures": 72
		},
		{
			"formatID": 120,
			"linearTilingFeatures": 3072,
			"optimalTilingFeatures": 3072,
			"bufferFeatures": 72
		},
		{
			"formatID": 122,
			"linearTilingFeatures": 54659,
			"optimalTilingFeatures": 54659,
			"bufferFeatures": 88
		},
		{
			"formatID": 123,
			"linearTilingFeatures": 54273,
			"optimalTilingFeatures": 54273,
			"bufferFeatures": 72
		},
		{
			"formatID": 124,
			"linearTilingFeatures": 0,
			"optimalTilingFeatures": 122369,
			"bufferFeatures": 0
		},
		{
			"formatID": 125,
			"linearTilingFeatures": 0,
			"optimalTilingFeatures": 122369,
			"bufferFeatures": 0
		},
		{
			"formatID": 126,
			"linearTilingFeatures": 0,
			"optimalTilingFeatures": 122369,
			"bufferFeatures": 0
		},
		{
			"formatID": 127,
			"linearTilingFeatures": 0,
			"optimalTilingFeatures": 56833,
			"bufferFeatures": 0
		},
		{
			"formatID": 129,
			"linearTilingFeatures": 0,
			"optimalTilingFeatures": 122369,
			"bufferFeatures": 0
		},
		{
			"formatID": 130,
			"linearTilingFeatures": 0,
			"optimalTilingFeatures": 122369,
			"bufferFeatures": 0
		},
		{
			"formatID": 131,
			"linearTilingFeatures": 54273,
			"optimalTilingFeatures": 54273,
			"bufferFeatures": 0
		},
		{
			"formatID": 132,
			"linearTilingFeatures": 54273,
			"optimalTilingFeatures": 54273,
			"bufferFeatures": 0
		},
		{
			"formatID": 133,
			"linearTilingFeatures": 54273,
			"optimalTilingFeatures": 54273,
			"bufferFeatures": 0
		},
		{
			"formatID": 134,
			"linearTilingFeatures": 54273,
			"optimalTilingFeatures": 54273,
			"bufferFeatures": 0
		},
		{
			"formatID": 135,
			"linearTilingFeatures": 54273,
			"optimalTilingFeatures": 54273,
			"bufferFeatures": 0
		},
		{
			"formatID": 136,
			"linearTilingFeatures": 54273,
			"optimalTilingFeatures": 54273,
			"bufferFeatures": 0
		},
		{
			"formatID": 137,
			"linearTilingFeatures": 54273,
			"optimalTilingFeatures": 54273,
			"bufferFeatures": 0
		},
		{
			"formatID": 138,
			"linearTilingFeatures": 54273,
			"optimalTilingFeatures": 54273,
			"bufferFeatures": 0
		},
		{
			"formatID": 139,
			"linearTilingFeatures": 54273,
			"optimalTilingFeatures": 54273,
			"bufferFeatures": 0
		},
		{
			"formatID": 140,
			"linearTilingFeatures": 54273,
			"optimalTilingFeatures": 54273,
			"bufferFeatures": 0
		},
		{
			"formatID": 141,
			"linearTilingFeatures": 54273,
			"optimalTilingFeatures": 54273,
			"bufferFeatures": 0
		},
		{
			"formatID": 142,
			"linearTilingFeatures": 54273,
			"optimalTilingFeatures": 54273,
			"bufferFeatures": 0
		},
		{
			"formatID": 143,
			"linearTilingFeatures": 54273,
			"optimalTilingFeatures": 54273,
			"bufferFeatures": 0
		},
		{
			"formatID": 144,
			"linearTilingFeatures": 54273,
			"optimalTilingFeatures": 54273,
			"bufferFeatures": 0
		},
		{
			"formatID": 145,
			"linearTilingFeatures": 54273,
			"optimalTilingFeatures": 54273,
			"bufferFeatures": 0
		},
		{
			"formatID": 146,
			"linearTilingFeatures": 54273,
			"optimalTilingFeatures": 54273,
			"bufferFeatures": 0
		},
		{
			"formatID": 1000156007,
			"linearTilingFeatures": 56707,
			"optimalTilingFeatures": 56707,
			"bufferFeatures": 88
		}
	]
}
    "#;

        let manifest: VulkanInfoDevSimManifest = serde_json::from_str(data).unwrap();
        let entry = SummaryDeviceEntry {
            api_version: "123",
            device_name: "test",
            driver_name: "test",
            driver_info: "test",
            driver_version: "test",
            vendor_id: "asd",
            device_id: "123",
        };
        let info = parse_legacy_devsim(manifest, &entry);
        assert_eq!(Some(&true), info.extensions.get("VK_KHR_device_group"));
        assert_eq!(Some(&false), info.extensions.get("VK_AMDX_shader_enqueue"));
    }
}
