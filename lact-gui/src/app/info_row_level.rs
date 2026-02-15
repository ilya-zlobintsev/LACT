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
    use std::cell::RefCell;

    use glib::Properties;
    use gtk::{LevelBar, glib, prelude::*, subclass::prelude::*};
    use relm4::view;

    use crate::app::animation::SpringAnimation;
    use crate::app::info_row::{InfoRow, InfoRowExt};

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::InfoRowLevel)]
    pub struct InfoRowLevel {
        #[property(get, set = Self::set_level_value)]
        level_value: RefCell<f64>,
        animation: RefCell<SpringAnimation<LevelBar>>,
    }

    impl InfoRowLevel {
        fn set_level_value(&self, value: f64) {
            let old = *self.level_value.borrow();
            if (old - value).abs() < f64::EPSILON {
                return;
            }

            self.level_value.replace(value);
            self.animation.borrow().animate_to(value);
        }
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
                        remove_offset_value: Some(gtk::LEVEL_BAR_OFFSET_LOW),
                        remove_offset_value: Some(gtk::LEVEL_BAR_OFFSET_HIGH),
                        remove_offset_value: Some(gtk::LEVEL_BAR_OFFSET_FULL),
                    }
                }
            }

            let animation = SpringAnimation::new(&level_bar, |bar, v| {
                bar.set_value(v);
            });
            self.animation.replace(animation);
        }
    }

    impl WidgetImpl for InfoRowLevel {}
    impl BoxImpl for InfoRowLevel {}
}
