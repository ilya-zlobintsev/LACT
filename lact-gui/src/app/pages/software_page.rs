mod vulkan;

use crate::{
    app::{format_friendly_size, info_row::InfoRow, page_section::PageSection},
    GUI_VERSION, REPO_URL,
};
use gtk::prelude::*;
use indexmap::IndexMap;
use lact_client::schema::{SystemInfo, GIT_COMMIT};
use lact_schema::DeviceInfo;
use relm4::{Component, ComponentController, ComponentParts, ComponentSender, RelmWidgetExt};
use std::{fmt::Write, sync::Arc};
use vulkan::feature_window::{VulkanFeature, VulkanFeaturesWindow};

pub struct SoftwarePage {
    device_info: Option<Arc<DeviceInfo>>,
}

#[derive(Debug)]
pub enum SoftwarePageMsg {
    DeviceInfo(Arc<DeviceInfo>),
    ShowVulkanFeatures,
    ShowVulkanExtensions,
}

#[relm4::component(pub)]
impl relm4::SimpleComponent for SoftwarePage {
    type Init = (SystemInfo, bool);
    type Input = SoftwarePageMsg;
    type Output = ();

    view! {
        gtk::ScrolledWindow {
            set_hscrollbar_policy: gtk::PolicyType::Never,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 15,
                set_margin_horizontal: 20,

                PageSection::new("System") {
                    append = &InfoRow::new_selectable("LACT Daemon:", &daemon_version),
                    append = &InfoRow::new_selectable("LACT GUI:", &gui_version),
                    append = &InfoRow::new_selectable("Kernel Version:", &system_info.kernel_version),
                },

                #[name = "vulkan_stack"]
                match model.device_info.as_ref().and_then(|info| info.vulkan_info.as_ref()) {
                    Some(info) => {
                        PageSection::new("Vulkan") {
                            append = &InfoRow {
                                set_name: "Device Name:",
                                #[watch]
                                set_value: info.device_name.as_str(),
                                set_selectable: true,
                            },
                            append = &InfoRow {
                                set_name: "API Version:",
                                #[watch]
                                set_value: info.api_version.as_str(),
                                set_selectable: true,
                            },
                            append = &InfoRow {
                                set_name: "Driver Name:",
                                #[watch]
                                set_value: info.driver.name.as_deref().unwrap_or_default(),
                                set_selectable: true,
                            },
                            append = &InfoRow {
                                set_name: "Driver Version:",
                                #[watch]
                                set_value: info.driver.info.as_deref().unwrap_or_default(),
                                set_selectable: true,
                            },

                            append = &gtk::Box {
                                set_orientation: gtk::Orientation::Horizontal,
                                set_hexpand: true,

                                append = &gtk::Label {
                                    set_halign: gtk::Align::Start,
                                    set_hexpand: true,
                                    set_label: "Features:"
                                },

                                append = &gtk::Button {
                                    connect_clicked => SoftwarePageMsg::ShowVulkanFeatures,
                                    set_label: "Show",
                                }
                            },

                            append = &gtk::Box {
                                set_orientation: gtk::Orientation::Horizontal,
                                set_hexpand: true,

                                append = &gtk::Label {
                                    set_halign: gtk::Align::Start,
                                    set_hexpand: true,
                                    set_label: "Extensions:"
                                },

                                append = &gtk::Button {
                                    connect_clicked => SoftwarePageMsg::ShowVulkanExtensions,
                                    set_label: "Show",
                                }
                            },
                        }
                    }
                    None => {
                        PageSection::new("Vulkan") {
                            append = &gtk::Label {
                                set_label: "Vulkan device not found",
                                set_halign: gtk::Align::Start,
                            },
                        }
                    }
                },

                #[name = "opencl_stack"]
                match model.device_info.as_ref().and_then(|info| info.opencl_info.as_ref()) {
                    Some(info) => {
                        PageSection::new("OpenCL") {
                            append = &InfoRow {
                                set_name: "Platform Name:",
                                #[watch]
                                set_value: info.platform_name.as_str(),
                                set_selectable: true,
                            },
                            append = &InfoRow {
                                set_name: "Device Name:",
                                #[watch]
                                set_value: info.device_name.as_str(),
                                set_selectable: true,
                            },
                            append = &InfoRow {
                                set_name: "Version:",
                                #[watch]
                                set_value: info.version.as_str(),
                                set_selectable: true,
                            },
                            append = &InfoRow {
                                set_name: "Driver Version:",
                                #[watch]
                                set_value: info.driver_version.as_str(),
                                set_selectable: true,
                            },
                            append = &InfoRow {
                                set_name: "OpenCL C Version:",
                                #[watch]
                                set_value: info.c_version.as_str(),
                                set_selectable: true,
                            },
                            append = &InfoRow {
                                set_name: "Compute Units:",
                                #[watch]
                                set_value: info.compute_units.to_string(),
                                set_selectable: true,
                            },
                            append = &InfoRow {
                                set_name: "Workgroup Size:",
                                #[watch]
                                set_value: info.workgroup_size.to_string(),
                                set_selectable: true,
                            },
                            append = &InfoRow {
                                set_name: "Global Memory:",
                                #[watch]
                                set_value: format_friendly_size(info.global_memory),
                                set_selectable: true,
                            },
                            append = &InfoRow {
                                set_name: "Local Memory:",
                                #[watch]
                                set_value: format_friendly_size(info.local_memory),
                                set_selectable: true,
                            },
                        }
                    }
                    None => {
                        PageSection::new("OpenCL") {
                            append = &gtk::Label {
                                set_label: "OpenCL device not found",
                                set_halign: gtk::Align::Start,
                            },
                        }
                    }
                },
            }
        },
    }

    fn init(
        (system_info, embedded): Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self { device_info: None };

        let mut daemon_version = format!("{}-{}", system_info.version, system_info.profile);
        if embedded {
            daemon_version.push_str("-embedded");
        }
        if let Some(commit) = &system_info.commit {
            let daemon_commit_link = format!("{REPO_URL}/commit/{commit}");
            write!(
                daemon_version,
                " (commit <a href=\"{daemon_commit_link}\">{commit}</a>)"
            )
            .unwrap();
        }

        let gui_profile = if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        };
        let gui_commit_link = format!("{REPO_URL}/commit/{GIT_COMMIT}");
        let gui_version = format!(
            "{GUI_VERSION}-{gui_profile} (commit <a href=\"{gui_commit_link}\">{GIT_COMMIT}</a>)"
        );

        let widgets = view_output!();

        widgets.vulkan_stack.set_vhomogeneous(false);
        widgets.opencl_stack.set_vhomogeneous(false);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            SoftwarePageMsg::DeviceInfo(info) => {
                self.device_info = Some(info);
            }
            SoftwarePageMsg::ShowVulkanFeatures => {
                if let Some(info) = &self.device_info {
                    if let Some(vulkan_info) = &info.vulkan_info {
                        show_features_window("Vulkan Features", &vulkan_info.features);
                    }
                }
            }
            SoftwarePageMsg::ShowVulkanExtensions => {
                if let Some(info) = &self.device_info {
                    if let Some(vulkan_info) = &info.vulkan_info {
                        show_features_window("Vulkan Extensions", &vulkan_info.extensions);
                    }
                }
            }
        }
    }
}

fn show_features_window(title: &str, values: &IndexMap<String, bool>) {
    let values = values
        .into_iter()
        .map(|(name, &supported)| VulkanFeature {
            name: name.clone(),
            supported,
        })
        .collect();

    let mut window_controller = VulkanFeaturesWindow::builder()
        .launch((values, title.to_owned()))
        .detach();
    window_controller.detach_runtime();
    window_controller.widget().present();
}
