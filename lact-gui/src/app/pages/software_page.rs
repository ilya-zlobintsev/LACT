mod vulkan;

use crate::{
    app::{format_friendly_size, info_row::InfoRow, page_section::PageSection},
    GUI_VERSION, I18N, REPO_URL,
};
use gtk::prelude::*;
use i18n_embed_fl::fl;
use indexmap::IndexMap;
use lact_client::schema::{SystemInfo, GIT_COMMIT};
use lact_schema::{DeviceInfo, VulkanInfo};
use relm4::{Component, ComponentController, ComponentParts, ComponentSender, RelmWidgetExt};
use relm4_components::simple_combo_box::{SimpleComboBox, SimpleComboBoxMsg};
use std::{fmt::Write, sync::Arc};
use vulkan::feature_window::{VulkanFeature, VulkanFeaturesWindow};

pub struct SoftwarePage {
    device_info: Option<Arc<DeviceInfo>>,

    vulkan_driver_selector: relm4::Controller<SimpleComboBox<String>>,
}

#[derive(Debug)]
pub enum SoftwarePageMsg {
    DeviceInfo(Arc<DeviceInfo>),
    ShowVulkanFeatures,
    ShowVulkanExtensions,
    SelectionChanged,
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

                PageSection::new(&fl!(I18N, "system-section")) {
                    append = &InfoRow::new_selectable(&fl!(I18N, "lact-daemon"), &daemon_version),
                    append = &InfoRow::new_selectable(&fl!(I18N, "lact-gui"), &gui_version),
                    append = &InfoRow::new_selectable(&fl!(I18N, "kernel-version"), &system_info.kernel_version),
                },

                #[name = "vulkan_stack"]
                match model.selected_vulkan_info() {
                    Some(info) => {
                        PageSection::new("Vulkan") {
                            append = &gtk::Box {
                                set_orientation: gtk::Orientation::Horizontal,
                                set_hexpand: true,
                                #[watch]
                                set_visible: model.vulkan_driver_selector.model().variants.len() > 1,

                                append = &gtk::Label {
                                    set_halign: gtk::Align::Start,
                                    set_hexpand: true,
                                    set_label: &fl!(I18N, "instance"),
                                },

                                append = model.vulkan_driver_selector.widget(),
                            },

                            append = &InfoRow {
                                set_name: fl!(I18N, "device-name"),
                                #[watch]
                                set_value: info.device_name.as_str(),
                                set_selectable: true,
                            },
                            append = &InfoRow {
                                set_name: fl!(I18N, "api-version"),
                                #[watch]
                                set_value: info.api_version.as_str(),
                                set_selectable: true,
                            },
                            append = &InfoRow {
                                set_name: fl!(I18N, "driver-name"),
                                #[watch]
                                set_value: info.driver.name.as_deref().unwrap_or_default(),
                                set_selectable: true,
                            },
                            append = &InfoRow {
                                set_name: fl!(I18N, "driver-version"),
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
                                    set_label: &format!("{}:", fl!(I18N, "features")),
                                },

                                append = &gtk::Button {
                                    connect_clicked => SoftwarePageMsg::ShowVulkanFeatures,
                                    set_label: &fl!(I18N, "show-button"),
                                }
                            },

                            append = &gtk::Box {
                                set_orientation: gtk::Orientation::Horizontal,
                                set_hexpand: true,

                                append = &gtk::Label {
                                    set_halign: gtk::Align::Start,
                                    set_hexpand: true,
                                    set_label: &format!("{}:", fl!(I18N, "extensions")),
                                },

                                append = &gtk::Button {
                                    connect_clicked => SoftwarePageMsg::ShowVulkanExtensions,
                                    set_label: &fl!(I18N, "show-button"),
                                }
                            },
                        }
                    }
                    None => {
                        PageSection::new("Vulkan") {
                            append = &gtk::Label {
                                set_label: &fl!(I18N, "device-not-found", kind = "Vulkan"),
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
                                set_name: fl!(I18N, "platform-name"),
                                #[watch]
                                set_value: info.platform_name.as_str(),
                                set_selectable: true,
                            },
                            append = &InfoRow {
                                set_name: fl!(I18N, "device-name"),
                                #[watch]
                                set_value: info.device_name.as_str(),
                                set_selectable: true,
                            },
                            append = &InfoRow {
                                set_name: fl!(I18N, "version"),
                                #[watch]
                                set_value: info.version.as_str(),
                                set_selectable: true,
                            },
                            append = &InfoRow {
                                set_name: fl!(I18N, "driver-version"),
                                #[watch]
                                set_value: info.driver_version.as_str(),
                                set_selectable: true,
                            },
                            append = &InfoRow {
                                set_name: fl!(I18N, "cl-c-version"),
                                #[watch]
                                set_value: info.c_version.as_str(),
                                set_selectable: true,
                            },
                            append = &InfoRow {
                                set_name: fl!(I18N, "compute-units"),
                                #[watch]
                                set_value: info.compute_units.to_string(),
                                set_selectable: true,
                            },
                            append = &InfoRow {
                                set_name: fl!(I18N, "workgroup-size"),
                                #[watch]
                                set_value: info.workgroup_size.to_string(),
                                set_selectable: true,
                            },
                            append = &InfoRow {
                                set_name: fl!(I18N, "global-memory"),
                                #[watch]
                                set_value: format_friendly_size(info.global_memory),
                                set_selectable: true,
                            },
                            append = &InfoRow {
                                set_name: fl!(I18N, "local-memory"),
                                #[watch]
                                set_value: format_friendly_size(info.local_memory),
                                set_selectable: true,
                            },
                        }
                    }
                    None => {
                        PageSection::new("OpenCL") {
                            append = &gtk::Label {
                                set_label: &fl!(I18N, "device-not-found", kind = "OpenCL"),
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
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let vulkan_driver_selector = SimpleComboBox::builder()
            .launch(SimpleComboBox {
                variants: vec![],
                active_index: None,
            })
            .forward(sender.input_sender(), |_| SoftwarePageMsg::SelectionChanged);

        let model = Self {
            device_info: None,
            vulkan_driver_selector,
        };

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
                let mut vulkan_drivers = Vec::new();

                for info in &info.vulkan_instances {
                    let name = format!(
                        "{} ({})",
                        info.device_name,
                        info.driver.name.as_deref().unwrap_or_default()
                    );
                    vulkan_drivers.push(name);
                }

                let selected_driver = if vulkan_drivers.is_empty() {
                    None
                } else {
                    Some(0)
                };
                self.vulkan_driver_selector
                    .emit(SimpleComboBoxMsg::UpdateData(SimpleComboBox {
                        variants: vulkan_drivers,
                        active_index: selected_driver,
                    }));

                self.device_info = Some(info);
            }
            SoftwarePageMsg::ShowVulkanFeatures => {
                if let Some(vulkan_info) = &self.selected_vulkan_info() {
                    show_features_window("Vulkan Features", &vulkan_info.features);
                }
            }
            SoftwarePageMsg::ShowVulkanExtensions => {
                if let Some(vulkan_info) = self.selected_vulkan_info() {
                    show_features_window("Vulkan Extensions", &vulkan_info.extensions);
                }
            }
            SoftwarePageMsg::SelectionChanged => (),
        }
    }
}

impl SoftwarePage {
    fn selected_vulkan_info(&self) -> Option<&VulkanInfo> {
        self.vulkan_driver_selector
            .model()
            .active_index
            .and_then(|idx| {
                self.device_info
                    .as_ref()
                    .and_then(|info| info.vulkan_instances.get(idx))
            })
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
