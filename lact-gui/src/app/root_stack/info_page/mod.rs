mod vulkan_info;

use gtk::prelude::*;
use gtk::*;
use lact_client::schema::{DeviceInfo, DeviceStats};
use vulkan_info::VulkanInfoFrame;

use super::{label_row, section_box, values_grid};

#[derive(Clone)]
pub struct InformationPage {
    pub container: ScrolledWindow,
    gpu_name_label: Label,
    gpu_manufacturer_label: Label,
    family_name: Label,
    asic_name: Label,
    vbios_version_label: Label,
    driver_label: Label,
    vram_size_label: Label,
    vram_type_label: Label,
    vram_peak_bw_label: Label,
    compute_units_label: Label,
    cpu_accessible_vram_label: Label,
    link_speed_label: Label,
    vulkan_info_frame: VulkanInfoFrame,
    vulkan_unavailable_label: Label,
}

impl InformationPage {
    pub fn new() -> Self {
        let vbox = Box::new(Orientation::Vertical, 15);

        let info_container = section_box("Hardware Information");

        let values_grid = values_grid();

        // Dummy label to prevent the gpu name label from stealing focus
        let dummy_label = Label::builder().selectable(true).halign(Align::End).build();
        values_grid.attach(&dummy_label, 0, 0, 1, 1);

        let gpu_name_label = label_row("GPU Model:", &values_grid, 0, 0, true);
        let gpu_manufacturer_label = label_row("GPU Manufacturer:", &values_grid, 1, 0, true);
        let family_name = label_row("GPU Family:", &values_grid, 2, 0, true);
        let asic_name = label_row("ASIC Name:", &values_grid, 3, 0, true);
        let compute_units_label = label_row("Compute Units:", &values_grid, 4, 0, true);
        let vbios_version_label = label_row("VBIOS Version:", &values_grid, 5, 0, true);
        let driver_label = label_row("Driver Used:", &values_grid, 6, 0, true);
        let vram_size_label = label_row("VRAM Size:", &values_grid, 7, 0, true);
        let vram_type_label = label_row("VRAM Type:", &values_grid, 8, 0, true);
        let vram_peak_bw_label = label_row("Peak VRAM Bandwidth:", &values_grid, 9, 0, true);
        let cpu_accessible_vram_label =
            label_row("CPU Accessible VRAM:", &values_grid, 10, 0, true);
        let link_speed_label = label_row("Link Speed:", &values_grid, 11, 0, true);

        info_container.append(&values_grid);
        vbox.append(&info_container);

        let vulkan_container = section_box("Vulkan Information");

        let vulkan_info_frame = VulkanInfoFrame::new();
        vulkan_container.append(&vulkan_info_frame.container);

        let vulkan_unavailable_label = Label::builder()
            .label("Vulkan is not available on this GPU")
            .visible(false)
            .margin_start(10)
            .margin_end(10)
            .halign(Align::Start)
            .build();
        vulkan_container.append(&vulkan_unavailable_label);

        vbox.append(&vulkan_container);

        let container = ScrolledWindow::builder()
            .hscrollbar_policy(PolicyType::Never)
            .child(&vbox)
            .build();

        Self {
            container,
            gpu_name_label,
            gpu_manufacturer_label,
            vbios_version_label,
            driver_label,
            vram_size_label,
            link_speed_label,
            vulkan_info_frame,
            family_name,
            asic_name,
            vram_type_label,
            cpu_accessible_vram_label,
            compute_units_label,
            vram_peak_bw_label,
            vulkan_unavailable_label,
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
            .set_markup(&format!("<b>{gpu_name}</b>"));

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
            .set_markup(&format!("<b>{gpu_manufacturer}</b>"));

        let mut family_name = "Unknown";
        let mut asic_name = "Unknown";
        let mut compute_units = "Unknown".to_owned();
        let mut vram_type = "Unknown";
        let mut vram_max_bw = "Unknown";
        let mut cpu_accessible_vram = "Unknown".to_owned();
        if let Some(drm_info) = &gpu_info.drm_info {
            family_name = &drm_info.family_name;
            asic_name = &drm_info.asic_name;
            compute_units = drm_info.compute_units.to_string();
            vram_type = &drm_info.vram_type;
            vram_max_bw = &drm_info.vram_max_bw;

            if let Some(memory_info) = &drm_info.memory_info {
                cpu_accessible_vram = (memory_info.cpu_accessible_total / 1024 / 1024).to_string();
            }
        }

        self.family_name
            .set_markup(&format!("<b>{family_name}</b>"));
        self.asic_name.set_markup(&format!("<b>{asic_name}</b>"));
        self.compute_units_label
            .set_markup(&format!("<b>{compute_units}</b>"));
        self.vram_type_label
            .set_markup(&format!("<b>{vram_type}</b>"));
        self.vram_peak_bw_label
            .set_markup(&format!("<b>{vram_max_bw} GiB/s</b>"));

        self.cpu_accessible_vram_label
            .set_markup(&format!("<b>{cpu_accessible_vram} MiB</b>"));

        let vbios_version = gpu_info.vbios_version.as_deref().unwrap_or("Unknown");
        self.vbios_version_label
            .set_markup(&format!("<b>{vbios_version}</b>"));

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
            self.vulkan_unavailable_label.hide();
        } else {
            self.vulkan_info_frame.container.hide();
            self.vulkan_unavailable_label.show();
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
