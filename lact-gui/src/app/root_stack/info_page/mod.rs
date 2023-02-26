mod vulkan_info;

use gtk::prelude::*;
use gtk::*;
use lact_client::schema::{DeviceInfo, DeviceStats};
use vulkan_info::VulkanInfoFrame;

use super::{label_row, section_box, values_grid};

#[derive(Clone)]
pub struct InformationPage {
    pub container: Box,
    gpu_name_label: Label,
    gpu_manufacturer_label: Label,
    vbios_version_label: Label,
    driver_label: Label,
    vram_size_label: Label,
    link_speed_label: Label,
    vulkan_info_frame: VulkanInfoFrame,
}

impl InformationPage {
    pub fn new() -> Self {
        let container = Box::new(Orientation::Vertical, 15);

        let info_container = section_box("Basic Information");

        let values_grid = values_grid();

        // Dummy label to prevent the gpu name label from stealing focus
        let dummy_label = Label::builder().selectable(true).halign(Align::End).build();
        values_grid.attach(&dummy_label, 0, 0, 1, 1);

        let gpu_name_label = label_row("GPU Model:", &values_grid, 0, 0, true);
        let gpu_manufacturer_label = label_row("GPU Manufacturer:", &values_grid, 1, 0, true);
        let vbios_version_label = label_row("VBIOS Version:", &values_grid, 2, 0, true);
        let driver_label = label_row("Driver Version:", &values_grid, 3, 0, true);
        let vram_size_label = label_row("VRAM Size:", &values_grid, 4, 0, true);
        let link_speed_label = label_row("Link Speed:", &values_grid, 5, 0, true);

        info_container.append(&values_grid);
        container.append(&info_container);

        let vulkan_container = section_box("Vulkan Information");

        let vulkan_info_frame = VulkanInfoFrame::new();

        vulkan_container.append(&vulkan_info_frame.container);
        container.append(&vulkan_container);

        Self {
            container,
            gpu_name_label,
            gpu_manufacturer_label,
            vbios_version_label,
            driver_label,
            vram_size_label,
            link_speed_label,
            vulkan_info_frame,
        }
    }

    pub fn set_info(&self, gpu_info: &DeviceInfo) {
        let gpu_name = gpu_info
            .pci_info
            .as_ref()
            .and_then(|pci_info| {
                pci_info
                    .subsystem_pci_info
                    .model
                    .as_deref()
                    .or(pci_info.device_pci_info.model.as_deref())
            })
            .unwrap_or_default();
        self.gpu_name_label
            .set_markup(&format!("<b>{gpu_name}</b>",));

        let gpu_manufacturer = gpu_info
            .pci_info
            .as_ref()
            .and_then(|pci_info| {
                pci_info
                    .subsystem_pci_info
                    .vendor
                    .as_deref()
                    .or(pci_info.device_pci_info.model.as_deref())
            })
            .unwrap_or_default();
        self.gpu_manufacturer_label
            .set_markup(&format!("<b>{gpu_manufacturer}</b>",));

        let vbios_version = gpu_info.vbios_version.as_deref().unwrap_or("Unknown");
        self.vbios_version_label
            .set_markup(&format!("<b>{vbios_version}</b>",));

        self.driver_label
            .set_markup(&format!("<b>{}</b>", gpu_info.driver));

        let link_speed = gpu_info
            .link_info
            .current_speed
            .as_deref()
            .unwrap_or("Unknown");
        let link_width = gpu_info
            .link_info
            .current_width
            .as_deref()
            .unwrap_or("Unknown");
        self.link_speed_label
            .set_markup(&format!("<b>{link_speed} x{link_width}</b>",));

        if let Some(vulkan_info) = &gpu_info.vulkan_info {
            self.vulkan_info_frame.set_info(vulkan_info);
            self.vulkan_info_frame.container.show();
        } else {
            self.vulkan_info_frame.container.hide();
        }
    }

    pub fn set_stats(&self, stats: &DeviceStats) {
        let vram_size = stats.vram.total.map_or_else(
            || "Unknown".to_owned(),
            |size| (size / 1024 / 1024).to_string(),
        );
        self.vram_size_label
            .set_markup(&format!("<b>{vram_size} MiB</b>"));
    }
}
