use anyhow::{anyhow, bail, Context};
use indexmap::{map::Entry, IndexMap};
use lact_schema::{GpuPciInfo, VulkanDriverInfo, VulkanInfo};
use serde::Deserialize;
use std::fs;
use tempfile::tempdir;
use tokio::process::Command;
use tracing::trace;

include!(concat!(env!("OUT_DIR"), "/vulkan_constants.rs"));

#[cfg_attr(test, allow(unreachable_code, unused_variables))]
pub async fn get_vulkan_info(pci_info: &GpuPciInfo) -> anyhow::Result<VulkanInfo> {
    #[cfg(test)]
    return Ok(VulkanInfo::default());

    let workdir = tempdir().context("Could not create temp folder")?;

    trace!("Reading vulkan info");
    let vendor_id = u32::from_str_radix(&pci_info.device_pci_info.vendor_id, 16)?;
    let device_id = u32::from_str_radix(&pci_info.device_pci_info.model_id, 16)?;
    trace!("Reading vulkan info");

    let summary_output = Command::new("vulkaninfo")
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
            let output = Command::new("vulkaninfo")
                .arg(format!("--json={i}"))
                .current_dir(workdir.path())
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

            let file_path = workdir.path().join(entry.file_name());
            let manifest = fs::read_to_string(&file_path).with_context(|| {
                format!("Could not read info file from '{}'", file_path.display())
            })?;

            return parse_manifest(&manifest, &entry);
        }
    }

    Err(anyhow!(
        "Could not find a vulkan device with matching pci ids"
    ))
}

fn parse_manifest(manifest: &str, entry: &SummaryDeviceEntry<'_>) -> anyhow::Result<VulkanInfo> {
    let manifest: VulkanInfoManifest =
        serde_json::from_str(manifest).context("Could not parse vulkan info file")?;

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

    let driver = VulkanDriverInfo {
        version: entry.driver_version.parse().unwrap_or(0),
        name: Some(entry.driver_name.to_owned()),
        info: Some(entry.driver_info.to_owned()),
        driver_version: None,
    };

    Ok(VulkanInfo {
        device_name: entry.device_name.to_owned(),
        api_version: entry.api_version.to_owned(),
        driver,
        enabled_layers: vec![],
        extensions,
        features,
    })
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

#[cfg(test)]
mod tests {
    use super::parse_summary;
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
}
