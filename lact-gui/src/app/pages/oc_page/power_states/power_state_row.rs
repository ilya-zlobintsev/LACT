use gtk::glib::{self, Object};
use lact_client::schema::PowerState;

glib::wrapper! {
    pub struct PowerStateRow(ObjectSubclass<imp::PowerStateRow>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable, gtk::Orientable;
}

impl PowerStateRow {
    pub fn new(power_state: PowerState, index: u8, value_suffix: &str) -> Self {
        let index = power_state.index.unwrap_or(index);

        let value_text = match power_state.min_value {
            Some(min) if min != power_state.value => format!("{min}-{}", power_state.value),
            _ => power_state.value.to_string(),
        };
        let title = format!("{index}: {value_text} {value_suffix}");
        Object::builder()
            .property("enabled", power_state.enabled)
            .property("title", title)
            .property("index", index)
            .build()
    }
}

mod imp {
    use gtk::{
        glib::{self, Properties},
        prelude::{BoxExt, ObjectExt, OrientableExt, WidgetExt},
        subclass::{prelude::*, widget::WidgetImpl},
    };
    use relm4::view;
    use std::{
        cell::RefCell,
        sync::atomic::{AtomicBool, AtomicU8},
    };

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::PowerStateRow)]
    pub struct PowerStateRow {
        #[property(get, set)]
        title: RefCell<String>,
        #[property(get, set)]
        enabled: AtomicBool,
        #[property(get, set)]
        index: AtomicU8,
        #[property(get, set)]
        active: AtomicBool,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PowerStateRow {
        const NAME: &'static str = "PowerStateRow";
        type Type = super::PowerStateRow;
        type ParentType = gtk::Box;
    }

    #[glib::derived_properties]
    impl ObjectImpl for PowerStateRow {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = &*self.obj();

            view! {
                #[local_ref]
                obj {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 5,

                    append: enabled_checkbutton = &gtk::CheckButton {
                        set_hexpand: true,
                    },

                    append: image = &gtk::Image {
                        set_icon_name: Some("pan-start-symbolic"),
                    },
                }
            };

            obj.bind_property("enabled", &enabled_checkbutton, "active")
                .bidirectional()
                .sync_create()
                .build();
            obj.bind_property("title", &enabled_checkbutton, "label")
                .bidirectional()
                .sync_create()
                .build();
            obj.bind_property("active", &image, "visible")
                .sync_create()
                .build();
        }
    }

    impl WidgetImpl for PowerStateRow {}
    impl BoxImpl for PowerStateRow {}
}
