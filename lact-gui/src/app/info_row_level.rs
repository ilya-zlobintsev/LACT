use crate::app::info_row::InfoRow;
use gtk::glib::{self, Object};

glib::wrapper! {
    pub struct InfoRowLevel(ObjectSubclass<imp::InfoRowLevel>)
        @extends InfoRow, gtk::Box, gtk::Widget,
        @implements gtk::Orientable, gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl InfoRowLevel {
    pub fn new(name: &str, value: &str, level: f64) -> Self {
        Object::builder()
            .property("name", name)
            .property("value", value)
            .property("level-value", level)
            .build()
    }
}

impl Default for InfoRowLevel {
    fn default() -> Self {
        Object::builder().build()
    }
}

mod imp {
    use crate::app::info_row::{InfoRow, InfoRowExt};
    use glib::Properties;
    use gtk::{LevelBar, glib, prelude::*, subclass::prelude::*};
    use relm4::view;
    use std::cell::RefCell;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::InfoRowLevel)]
    pub struct InfoRowLevel {
        #[property(get, set)]
        level_value: RefCell<f64>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for InfoRowLevel {
        const NAME: &'static str = "InfoRowLevel";
        type Type = super::InfoRowLevel;
        type ParentType = InfoRow;
    }

    #[glib::derived_properties]
    impl ObjectImpl for InfoRowLevel {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            view! {
                #[local_ref]
                obj {
                    #[name(level_bar)]
                    append_child = &LevelBar {
                        set_hexpand: true,
                        set_orientation: gtk::Orientation::Horizontal,
                        // this prevents re-colour of the bar when the value is close to 100%
                        remove_offset_value: Some(gtk::LEVEL_BAR_OFFSET_LOW),
                        remove_offset_value: Some(gtk::LEVEL_BAR_OFFSET_HIGH),
                        remove_offset_value: Some(gtk::LEVEL_BAR_OFFSET_FULL),
                    }
                }
            }

            obj.bind_property("level-value", &level_bar, "value")
                .sync_create()
                .build();
        }
    }

    impl WidgetImpl for InfoRowLevel {}
    impl BoxImpl for InfoRowLevel {}
}
