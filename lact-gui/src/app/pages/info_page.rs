mod hardware_info;
mod vulkan_info;

use self::hardware_info::HardwareInfoSection;
use super::{values_grid, PageUpdate};
use crate::app::page_section::PageSection;
use gtk::prelude::*;
use relm4::{Component, ComponentParts, ComponentSender, RelmWidgetExt};
use vulkan_info::VulkanInfoFrame;

pub struct InformationPage {
    hardware_info: HardwareInfoSection,
    vulkan_info: VulkanInfoFrame,
}

#[relm4::component(pub)]
impl Component for InformationPage {
    type Init = ();
    type Input = PageUpdate;
    type Output = ();
    type CommandOutput = ();

    view! {
        gtk::ScrolledWindow {
            set_hscrollbar_policy: gtk::PolicyType::Never,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 15,
                set_margin_horizontal: 20,

                model.hardware_info.clone(),

                #[name = "vulkan_section"]
                PageSection::new("Vulkan Information") -> PageSection {
                    set_spacing: 10,
                    set_margin_start: 15,

                    append = &model.vulkan_info.container.clone(),
                },

                #[name = "vulkan_unavailable_label"]
                gtk::Label {
                    set_label: "Vulkan is not available on this GPU",
                    set_visible: false,
                    set_margin_horizontal: 10,
                    set_halign: gtk::Align::Start,
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let hardware_info = HardwareInfoSection::new();
        let vulkan_info = VulkanInfoFrame::new();

        let model = Self {
            hardware_info,
            vulkan_info,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: Self::Input,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            PageUpdate::Info(gpu_info) => {
                self.hardware_info.set_info(&gpu_info);

                if let Some(vulkan_info) = &gpu_info.vulkan_info {
                    self.vulkan_info.set_info(vulkan_info);
                    self.vulkan_info.container.show();
                    widgets.vulkan_unavailable_label.hide();
                } else {
                    self.vulkan_info.container.hide();
                    widgets.vulkan_unavailable_label.show();
                }
            }
            PageUpdate::Stats(stats) => {
                self.hardware_info.set_stats(&stats);
            }
        }
    }
}
