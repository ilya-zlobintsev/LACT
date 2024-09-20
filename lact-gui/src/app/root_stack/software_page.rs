use crate::{app::info_row::InfoRow, GUI_VERSION};
use gtk::prelude::*;
use lact_client::schema::{SystemInfo, GIT_COMMIT};
use relm4::{ComponentParts, ComponentSender, SimpleComponent};
use std::fmt::Write;

pub struct SoftwarePage {}

#[relm4::component(pub)]
impl SimpleComponent for SoftwarePage {
    type Init = (SystemInfo, bool);
    type Input = ();
    type Output = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 10,
            set_margin_start: 5,
            set_margin_end: 5,
            set_margin_top: 5,
            set_margin_bottom: 5,

            append = &InfoRow::new_selectable("LACT Daemon:", &daemon_version),
            append = &InfoRow::new_selectable("LACT GUI:", &gui_version),
            append = &InfoRow::new_selectable("Kernel Version:", &system_info.kernel_version),
        }
    }

    fn init(
        (system_info, embedded): Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {};

        let mut daemon_version = format!("{}-{}", system_info.version, system_info.profile);
        if embedded {
            daemon_version.push_str("-embedded");
        }
        if let Some(commit) = system_info.commit {
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
}
