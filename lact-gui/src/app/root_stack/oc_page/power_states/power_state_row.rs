use gtk::glib::{self, Object};
use lact_client::schema::PowerState;
use std::fmt::Display;

glib::wrapper! {
    pub struct PowerStateRow(ObjectSubclass<imp::PowerStateRow>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable;
}

impl PowerStateRow {
    pub fn new<T: Display>(power_state: PowerState<T>, index: u8, value_suffix: &str) -> Self {
        let title = format!("{}: {} {}", index, power_state.value, value_suffix);
        Object::builder()
            .property("enabled", power_state.enabled)
            .property("title", title)
            .property("index", index)
            .build()
    }
}

mod imp {
    use gtk::{
        glib::{self, subclass::InitializingObject, Properties},
        prelude::ObjectExt,
        subclass::{
            prelude::*,
            widget::{CompositeTemplateClass, WidgetImpl},
        },
        CompositeTemplate,
    };
    use std::{
        cell::RefCell,
        sync::atomic::{AtomicBool, AtomicU8},
    };

    #[derive(CompositeTemplate, Default, Properties)]
    #[properties(wrapper_type = super::PowerStateRow)]
    #[template(file = "ui/oc_page/power_state_row.blp")]
    pub struct PowerStateRow {
        #[property(get, set)]
        title: RefCell<String>,
        #[property(get, set)]
        enabled: AtomicBool,
        #[property(get, set)]
        index: AtomicU8,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PowerStateRow {
        const NAME: &'static str = "PowerStateRow";
        type Type = super::PowerStateRow;
        type ParentType = gtk::Box;

        fn class_init(class: &mut Self::Class) {
            class.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for PowerStateRow {}

    impl WidgetImpl for PowerStateRow {}
    impl BoxImpl for PowerStateRow {}
}
