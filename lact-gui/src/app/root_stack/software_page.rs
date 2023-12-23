use crate::GUI_VERSION;
use gtk::glib::{self, Object};
use lact_client::schema::{SystemInfo, GIT_COMMIT};
use std::fmt::Write;

glib::wrapper! {
    pub struct SoftwarePage(ObjectSubclass<imp::SoftwarePage>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Orientable, gtk::Accessible, gtk::Buildable;
}

impl SoftwarePage {
    pub fn new(system_info: SystemInfo, embedded: bool) -> Self {
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

        Object::builder()
            .property("daemon-version", daemon_version)
            .property("gui-version", gui_version)
            .property("kernel-version", system_info.kernel_version)
            .build()
    }
}

mod imp {
    #![allow(clippy::enum_variant_names)]
    use crate::app::{info_row::InfoRow, page_section::PageSection};
    use glib::Properties;
    use gtk::{
        glib::{self, subclass::InitializingObject},
        prelude::*,
        subclass::{
            prelude::*,
            widget::{CompositeTemplateClass, WidgetImpl},
        },
        CompositeTemplate,
    };
    use std::cell::RefCell;

    #[derive(CompositeTemplate, Default, Properties)]
    #[properties(wrapper_type = super::SoftwarePage)]
    #[template(file = "ui/software_page.blp")]
    pub struct SoftwarePage {
        #[property(get, set)]
        daemon_version: RefCell<String>,
        #[property(get, set)]
        gui_version: RefCell<String>,
        #[property(get, set)]
        kernel_version: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SoftwarePage {
        const NAME: &'static str = "SoftwarePage";
        type Type = super::SoftwarePage;
        type ParentType = gtk::Box;

        fn class_init(class: &mut Self::Class) {
            InfoRow::ensure_type();
            PageSection::ensure_type();

            class.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for SoftwarePage {}

    impl WidgetImpl for SoftwarePage {}
    impl BoxImpl for SoftwarePage {}
}
