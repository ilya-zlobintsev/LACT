mod vulkan_info;

use super::LabelRow;
use crate::app::page_section::PageSection;
use gtk::prelude::*;
use gtk::*;
use lact_client::schema::{DeviceInfo, DeviceStats};
use vulkan_info::VulkanInfoFrame;

#[derive(Clone)]
pub struct InformationPage {
    pub container: ScrolledWindow,
    gpu_name_row: LabelRow,
    gpu_manufacturer_row: LabelRow,
    family_name_row: LabelRow,
    asic_name_row: LabelRow,
    vbios_version_row: LabelRow,
    driver_row: LabelRow,
    vram_size_row: LabelRow,
    vram_type_row: LabelRow,
    vram_peak_bw_row: LabelRow,
    compute_units_row: LabelRow,
    l1_cache_row: LabelRow,
    l2_cache_row: LabelRow,
    l3_cache_row: LabelRow,
    resizable_bar_enabled_row: LabelRow,
    cpu_accessible_vram_row: LabelRow,
    link_speed_row: LabelRow,
    vulkan_info_frame: VulkanInfoFrame,
    vulkan_unavailable_label: Label,
}

impl InformationPage {
    pub fn new() -> Self {
        let vbox = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(12)
            .build();

        let info_container = PageSection::new("Hardware Information");

        let info_listbox = ListBox::builder()
            .css_classes(["boxed-list"])
            .selection_mode(SelectionMode::None)
            .build();

        let gpu_name_row = LabelRow::new("GPU Model");
        let gpu_manufacturer_row = LabelRow::new("GPU Manufacturer");
        let family_name_row = LabelRow::new("GPU Family");
        let asic_name_row = LabelRow::new("ASIC Name");
        let compute_units_row = LabelRow::new("Compute Units");
        let vbios_version_row = LabelRow::new("VBIOS Version");
        let driver_row = LabelRow::new("Driver Used");

        let vram_size_row = LabelRow::new("VRAM Size");
        let vram_type_row = LabelRow::new("VRAM Type");
        let vram_peak_bw_row = LabelRow::new("Peak VRAM Bandwidth");

        let l1_cache_row = LabelRow::new("L1 Cache (Per CU)");
        let l2_cache_row = LabelRow::new("L2 Cache");
        let l3_cache_row = LabelRow::new("L3 Cache");

        let resizable_bar_enabled_row = LabelRow::new("Resizeable BAR");
        let cpu_accessible_vram_row = LabelRow::new("CPU Accessible VRAM");
        let link_speed_row = LabelRow::new("Link Speed");

        info_listbox.append(&gpu_name_row.container);
        info_listbox.append(&gpu_manufacturer_row.container);
        info_listbox.append(&family_name_row.container);
        info_listbox.append(&asic_name_row.container);
        info_listbox.append(&compute_units_row.container);
        info_listbox.append(&vbios_version_row.container);
        info_listbox.append(&driver_row.container);
        info_listbox.append(&vram_size_row.container);
        info_listbox.append(&vram_type_row.container);
        info_listbox.append(&vram_peak_bw_row.container);
        info_listbox.append(&l1_cache_row.container);
        info_listbox.append(&l2_cache_row.container);
        info_listbox.append(&l3_cache_row.container);
        info_listbox.append(&resizable_bar_enabled_row.container);
        info_listbox.append(&cpu_accessible_vram_row.container);
        info_listbox.append(&link_speed_row.container);
        info_container.append(&info_listbox);
        vbox.append(&info_container);

        let vulkan_container = PageSection::new("Vulkan Information");

        let vulkan_info_frame = VulkanInfoFrame::new();
        vulkan_container.append(&vulkan_info_frame.container);

        let vulkan_unavailable_label = Label::builder()
            .label("Vulkan is not available on this GPU")
            .css_classes(["error"])
            .visible(false)
            .xalign(0.0)
            .build();
        vulkan_container.append(&vulkan_unavailable_label);

        vbox.append(&vulkan_container);

        let clamp = libadwaita::Clamp::builder()
            .maximum_size(600)
            .margin_top(24)
            .margin_bottom(24)
            .child(&vbox)
            .build();

        let container = ScrolledWindow::builder()
            .hscrollbar_policy(PolicyType::Never)
            .child(&clamp)
            .build();

        Self {
            container,
            gpu_name_row,
            gpu_manufacturer_row,
            vbios_version_row,
            driver_row,
            vram_size_row,
            link_speed_row,
            vulkan_info_frame,
            family_name_row,
            asic_name_row,
            vram_type_row,
            resizable_bar_enabled_row,
            cpu_accessible_vram_row,
            compute_units_row,
            vram_peak_bw_row,
            l1_cache_row,
            l2_cache_row,
            l3_cache_row,
            vulkan_unavailable_label,
        }
    }

    pub fn set_info(&self, gpu_info: &DeviceInfo) {
        self.gpu_name_row.set_content(
            gpu_info
                .pci_info
                .as_ref()
                .and_then(|pci_info| {
                    pci_info
                        .subsystem_pci_info
                        .model
                        .as_deref()
                        .or(pci_info.device_pci_info.model.as_deref())
                })
                .unwrap_or_default(),
        );

        self.gpu_manufacturer_row.set_content(
            gpu_info
                .pci_info
                .as_ref()
                .and_then(|pci_info| {
                    pci_info
                        .subsystem_pci_info
                        .vendor
                        .as_deref()
                        .or(pci_info.device_pci_info.model.as_deref())
                })
                .unwrap_or_default(),
        );

        let mut family_name = "Unknown";
        let mut asic_name = "Unknown";
        let mut compute_units = "Unknown".to_owned();
        let mut vram_type = "Unknown";
        let mut vram_max_bw = "Unknown";
        let mut cpu_accessible_vram = "Unknown".to_owned();
        let mut resizeable_bar_enabled = "Unknown";
        let mut l1_cache = "Unknown".to_owned();
        let mut l2_cache = "Unknown".to_owned();
        let mut l3_cache = "Unknown".to_owned();

        if let Some(drm_info) = &gpu_info.drm_info {
            family_name = &drm_info.family_name;
            asic_name = &drm_info.asic_name;
            compute_units = drm_info.compute_units.to_string();
            vram_type = &drm_info.vram_type;
            vram_max_bw = &drm_info.vram_max_bw;
            l1_cache = format!("{} KiB", drm_info.l1_cache_per_cu / 1024);
            l2_cache = format!("{} KiB", drm_info.l2_cache / 1024);
            l3_cache = format!("{} MiB", drm_info.l3_cache_mb);

            if let Some(memory_info) = &drm_info.memory_info {
                resizeable_bar_enabled = if memory_info.resizeable_bar {
                    "Enabled"
                } else {
                    "Disabled"
                };

                cpu_accessible_vram = (memory_info.cpu_accessible_total / 1024 / 1024).to_string();
            }
        }

        self.family_name_row.set_content(family_name);
        self.asic_name_row.set_content(asic_name);
        self.compute_units_row.set_content(&compute_units);
        self.vbios_version_row
            .set_content(gpu_info.vbios_version.as_deref().unwrap_or("Unknown"));
        self.driver_row.set_content(gpu_info.driver);
        self.vram_type_row.set_content(vram_type);
        self.vram_peak_bw_row
            .set_content(&format!("{vram_max_bw} GiB/s"));
        self.l1_cache_row.set_content(&l1_cache);
        self.l2_cache_row.set_content(&l2_cache);
        self.l3_cache_row.set_content(&l3_cache);
        self.resizable_bar_enabled_row
            .set_content(resizeable_bar_enabled);
        self.cpu_accessible_vram_row
            .set_content(&cpu_accessible_vram);
        self.link_speed_row.set_content(&format!(
            "{link_speed} x{link_width}",
            link_speed = gpu_info
                .link_info
                .current_speed
                .as_deref()
                .unwrap_or("Unknown"),
            link_width = gpu_info
                .link_info
                .current_width
                .as_deref()
                .unwrap_or("Unknown")
        ));

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
        self.vram_size_row.set_content(&format!("{vram_size} MiB"));
    }
}
