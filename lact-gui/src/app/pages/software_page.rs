use crate::app::ext::FlowBoxExt;
mod vulkan;

use crate::{
    app::{
        formatting,
        info_row::{InfoRow, InfoRowExt},
        page_section::PageSection,
    },
    GUI_VERSION, I18N, REPO_URL,
};
use gtk::prelude::*;
use i18n_embed_fl::fl;
use indexmap::IndexMap;
use lact_client::schema::{SystemInfo, GIT_COMMIT};
use lact_schema::{DeviceInfo, OpenCLInfo, VulkanInfo};
use relm4::{Component, ComponentController, ComponentParts, ComponentSender, RelmWidgetExt};
use relm4_components::simple_combo_box::{SimpleComboBox, SimpleComboBoxMsg};
use std::{fmt::Write, sync::Arc};
use vulkan::feature_window::{VulkanFeature, VulkanFeaturesWindow};

pub struct SoftwarePage {
    device_info: Option<Arc<DeviceInfo>>,

    vulkan_driver_selector: relm4::Controller<SimpleComboBox<String>>,
    opencl_platform_selector: relm4::Controller<SimpleComboBox<String>>,

    vulkan_window: Option<relm4::Controller<VulkanFeaturesWindow>>,
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
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 15,
            set_margin_vertical: 15,
            set_margin_horizontal: 30,

            PageSection::new(&fl!(I18N, "system-section")) {
                append_child = &gtk::FlowBox {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_column_spacing: 10,
                    set_homogeneous: true,
                    set_min_children_per_line: 2,
                    set_max_children_per_line: 4,
                    set_selection_mode: gtk::SelectionMode::None,

                    append_child = &InfoRow::new_selectable(&fl!(I18N, "lact-daemon"), &daemon_version),
                    append_child = &InfoRow::new_selectable(&fl!(I18N, "lact-gui"), &gui_version),
                    append_child = &InfoRow::new_selectable(&fl!(I18N, "kernel-version"), &system_info.kernel_version),
                },
            },

            #[name = "vulkan_stack"]
            match model.selected_vulkan_info() {
                Some(info) => {
                    PageSection::new("Vulkan") {
                        append_child = &gtk::FlowBox {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_column_spacing: 10,
                            set_homogeneous: true,
                            set_min_children_per_line: 2,
                            set_max_children_per_line: 4,
                            set_selection_mode: gtk::SelectionMode::None,

                            append_child = &InfoRow {
                                set_name: fl!(I18N, "instance"),
                                append_child = model.vulkan_driver_selector.widget(),
                            } -> vulkan_instance_item: gtk::FlowBoxChild {
                                #[watch]
                                set_visible: model.vulkan_driver_selector.model().variants.len() > 1,
                            },

                            append_child = &InfoRow {
                                set_name: fl!(I18N, "device-name"),
                                #[watch]
                                set_value: info.device_name.as_str(),
                                set_selectable: true,
                            },
                            append_child = &InfoRow {
                                set_name: fl!(I18N, "api-version"),
                                #[watch]
                                set_value: info.api_version.as_str(),
                                set_selectable: true,
                            },
                            append_child = &InfoRow {
                                set_name: fl!(I18N, "driver-name"),
                                #[watch]
                                set_value: info.driver.name.as_deref().unwrap_or_default(),
                                set_selectable: true,
                            },
                            append_child = &InfoRow {
                                set_name: fl!(I18N, "driver-version"),
                                #[watch]
                                set_value: info.driver.info.as_deref().unwrap_or_default(),
                                set_selectable: true,
                            },

                            append_child = &InfoRow {
                                set_value: fl!(I18N, "features"),
                                set_icon: "go-next-symbolic".to_string(),
                                connect_clicked => SoftwarePageMsg::ShowVulkanFeatures,
                            },

                            append_child = &InfoRow {
                                set_value: fl!(I18N, "extensions"),
                                set_icon: "go-next-symbolic".to_string(),
                                connect_clicked => SoftwarePageMsg::ShowVulkanExtensions,
                            },
                        },
                    }
                }
                None => {
                    PageSection::new("Vulkan") {
                        append_child = &gtk::Label {
                            set_label: &fl!(I18N, "device-not-found", kind = "Vulkan"),
                            set_halign: gtk::Align::Start,
                        },
                    }
                }
            },

            #[name = "opencl_stack"]
            match model.selected_opencl_info() {
                Some(info) => {
                    PageSection::new("OpenCL") {
                        append_child = &gtk::FlowBox {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_column_spacing: 10,
                            set_homogeneous: true,
                            set_min_children_per_line: 2,
                            set_max_children_per_line: 4,
                            set_selection_mode: gtk::SelectionMode::None,

                            append_child = &InfoRow {
                                set_name: fl!(I18N, "platform-name"),
                                append_child = model.opencl_platform_selector.widget(),
                            } -> opencl_platform_item: gtk::FlowBoxChild {
                                #[watch]
                                set_visible: model.opencl_platform_selector.model().variants.len() > 1,
                            },

                            append_child = &InfoRow {
                                set_name: fl!(I18N, "platform-name"),
                                #[watch]
                                set_value: info.platform_name.as_str(),
                                set_selectable: true,
                            } -> opencl_platform_name_item: gtk::FlowBoxChild {
                                #[watch]
                                set_visible: model.opencl_platform_selector.model().variants.len() == 1,
                            },
                            append_child = &InfoRow {
                                set_name: fl!(I18N, "device-name"),
                                #[watch]
                                set_value: info.device_name.as_str(),
                                set_selectable: true,
                            },
                            append_child = &InfoRow {
                                set_name: fl!(I18N, "version"),
                                #[watch]
                                set_value: info.version.as_str(),
                                set_selectable: true,
                            },
                            append_child = &InfoRow {
                                set_name: fl!(I18N, "driver-version"),
                                #[watch]
                                set_value: info.driver_version.as_str(),
                                set_selectable: true,
                            },
                            append_child = &InfoRow {
                                set_name: fl!(I18N, "cl-c-version"),
                                #[watch]
                                set_value: info.c_version.as_str(),
                                set_selectable: true,
                            },
                            append_child = &InfoRow {
                                set_name: fl!(I18N, "compute-units"),
                                #[watch]
                                set_value: info.compute_units.to_string(),
                                set_selectable: true,
                            },
                            append_child = &InfoRow {
                                set_name: fl!(I18N, "workgroup-size"),
                                #[watch]
                                set_value: info.workgroup_size.to_string(),
                                set_selectable: true,
                            },
                            append_child = &InfoRow {
                                set_name: fl!(I18N, "global-memory"),
                                #[watch]
                                set_value: formatting::fmt_human_bytes(info.global_memory, None),
                                set_selectable: true,
                            },
                            append_child = &InfoRow {
                                set_name: fl!(I18N, "local-memory"),
                                #[watch]
                                set_value: formatting::fmt_human_bytes(info.local_memory, None),
                                set_selectable: true,
                            },
                        },
                    }
                }
                None => {
                    PageSection::new("OpenCL") {
                        append_child = &gtk::Label {
                            set_label: &fl!(I18N, "device-not-found", kind = "OpenCL"),
                            set_halign: gtk::Align::Start,
                        },
                    }
                }
            },
        }
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

        let opencl_platform_selector = SimpleComboBox::builder()
            .launch(SimpleComboBox {
                variants: vec![],
                active_index: None,
            })
            .forward(sender.input_sender(), |_| SoftwarePageMsg::SelectionChanged);

        let model = Self {
            device_info: None,
            vulkan_driver_selector,
            opencl_platform_selector,
            vulkan_window: None,
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

                let mut opencl_platforms = Vec::new();

                for info in &info.opencl_instances {
                    opencl_platforms.push(info.platform_name.clone());
                }

                let selected_platform = if opencl_platforms.is_empty() {
                    None
                } else {
                    Some(0)
                };
                self.opencl_platform_selector
                    .emit(SimpleComboBoxMsg::UpdateData(SimpleComboBox {
                        variants: opencl_platforms,
                        active_index: selected_platform,
                    }));

                self.device_info = Some(info);
            }
            SoftwarePageMsg::ShowVulkanFeatures => {
                if let Some(vulkan_info) = &self.selected_vulkan_info() {
                    self.vulkan_window = Some(show_features_window(
                        "Vulkan Features",
                        &vulkan_info.features,
                    ));
                }
            }
            SoftwarePageMsg::ShowVulkanExtensions => {
                if let Some(vulkan_info) = self.selected_vulkan_info() {
                    self.vulkan_window = Some(show_features_window(
                        "Vulkan Extensions",
                        &vulkan_info.extensions,
                    ));
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

    fn selected_opencl_info(&self) -> Option<&OpenCLInfo> {
        self.opencl_platform_selector
            .model()
            .active_index
            .and_then(|idx| {
                self.device_info
                    .as_ref()
                    .and_then(|info| info.opencl_instances.get(idx))
            })
    }
}

fn show_features_window(
    title: &str,
    values: &IndexMap<String, bool>,
) -> relm4::Controller<VulkanFeaturesWindow> {
    let values = values
        .into_iter()
        .map(|(name, &supported)| VulkanFeature {
            name: name.clone(),
            supported,
        })
        .collect();

    let window_controller = VulkanFeaturesWindow::builder()
        .launch((values, title.to_owned()))
        .detach();
    window_controller.widget().present();
    window_controller
}
