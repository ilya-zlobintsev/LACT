use super::PageUpdate;
use crate::{
    app::{format_friendly_size, info_row::InfoRow, page_section::PageSection},
    GUI_VERSION,
};
use gtk::prelude::*;
use lact_client::schema::{SystemInfo, GIT_COMMIT};
use lact_schema::DeviceInfo;
use relm4::{ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent};
use std::{fmt::Write, rc::Rc, sync::Arc};

pub struct SoftwarePage {
    device_info: Option<Arc<DeviceInfo>>,
}

#[relm4::component(pub)]
impl SimpleComponent for SoftwarePage {
    type Init = (Rc<SystemInfo>, bool);
    type Input = PageUpdate;
    type Output = ();

    view! {
        gtk::ScrolledWindow {
            set_hscrollbar_policy: gtk::PolicyType::Never,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 15,
                set_margin_horizontal: 20,

                PageSection::new("System") {
                    set_spacing: 10,

                    append = &InfoRow::new_selectable("LACT Daemon:", &daemon_version),
                    append = &InfoRow::new_selectable("LACT GUI:", &gui_version),
                    append = &InfoRow::new_selectable("Kernel Version:", &system_info.kernel_version),
                },

                match model.device_info.as_ref().and_then(|info| info.opencl_info.as_ref()) {
                    Some(info) => {
                        PageSection::new("OpenCL") {
                            set_spacing: 10,

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
                                set_name: "Compute Units:",
                                #[watch]
                                set_value: info.compute_units.to_string(),
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
            write!(daemon_version, " (commit {commit})").unwrap();
        }

        let gui_profile = if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        };
        let gui_version = format!("{GUI_VERSION}-{gui_profile} (commit {GIT_COMMIT})");

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            PageUpdate::Info(info) => {
                self.device_info = Some(info);
            }
            PageUpdate::Stats(_) => (),
        }
    }
}
