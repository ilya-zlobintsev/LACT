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

    use adw::prelude::*;
    use glib::Properties;
    use gtk::{LevelBar, glib, subclass::prelude::*};
    use relm4::view;

    use crate::app::info_row::{InfoRow, InfoRowExt};

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::InfoRowLevel)]
    pub struct InfoRowLevel {
        #[property(get, set = Self::set_level_value)]
        level_value: RefCell<f64>,
        animation: RefCell<Option<adw::SpringAnimation>>,
    }

    impl InfoRowLevel {
        fn set_level_value(&self, value: f64) {
            let clamped = value.clamp(0.0, 1.0);
            let rounded = (clamped * 100.0).round() / 100.0;

            let old = *self.level_value.borrow();
            if (old - rounded).abs() < f64::EPSILON {
                return;
            }

            self.level_value.replace(rounded);

            if let Some(animation) = self.animation.borrow().as_ref() {
                animation.set_value_from(old);
                animation.set_value_to(rounded);
                animation.play();
            }
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
                        set_overflow: gtk::Overflow::Hidden,
                        add_css_class: "info-row-level-bar",
                        // this prevents re-colour of the bar when the value is close to 100%
                        remove_offset_value: Some(gtk::LEVEL_BAR_OFFSET_LOW),
                        remove_offset_value: Some(gtk::LEVEL_BAR_OFFSET_HIGH),
                        remove_offset_value: Some(gtk::LEVEL_BAR_OFFSET_FULL),
                    }
                }
            }

            let animation_target = adw::CallbackAnimationTarget::new(glib::clone!(
                #[weak]
                level_bar,
                move |value| {
                    level_bar.set_value(value);
                }
            ));
            let spring_params = adw::SpringParams::new(1.0, 1.0, 800.0);
            let animation = adw::SpringAnimation::new(&level_bar, 0.0, 0.0, spring_params, animation_target);
            animation.set_clamp(true);
            self.animation.replace(Some(animation));
        }
    }

    impl WidgetImpl for InfoRowLevel {}
    impl BoxImpl for InfoRowLevel {}
}
