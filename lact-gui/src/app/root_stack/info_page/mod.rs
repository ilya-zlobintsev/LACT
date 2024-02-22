mod hardware_info;
mod vulkan_info;

use self::hardware_info::HardwareInfoSection;

use super::values_grid;
use crate::app::page_section::PageSection;
use gtk::prelude::*;
use gtk::*;
use lact_client::schema::{DeviceInfo, DeviceStats};
use vulkan_info::VulkanInfoFrame;

#[derive(Clone)]
pub struct InformationPage {
    pub container: ScrolledWindow,
    hardware_info: HardwareInfoSection,
    vulkan_info_frame: VulkanInfoFrame,
    vulkan_unavailable_label: Label,
}

impl InformationPage {
    pub fn new() -> Self {
        let vbox = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(15)
            .margin_start(20)
            .margin_end(20)
            .build();

        let hardware_info = HardwareInfoSection::new();
        vbox.append(&hardware_info);

        let vulkan_container = PageSection::new("Vulkan Information");

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
            hardware_info,
            vulkan_info_frame,
            vulkan_unavailable_label,
        }
    }

    pub fn set_info(&self, gpu_info: &DeviceInfo) {
        self.hardware_info.set_info(gpu_info);

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
        self.hardware_info.set_stats(stats);
    }
}
