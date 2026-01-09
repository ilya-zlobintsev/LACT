use gtk::glib::{self, Object};

glib::wrapper! {
    pub struct InfoRowLevel(ObjectSubclass<imp::InfoRowLevel>)
        @extends gtk::Box, gtk::Widget,
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

    pub fn set_value_size_group(&self, size_group: &gtk::SizeGroup) {
        use glib::subclass::types::ObjectSubclassIsExt;
        size_group.add_widget(&self.imp().value_label);
    }
}

impl Default for InfoRowLevel {
    fn default() -> Self {
        Object::builder().build()
    }
}

mod imp {
    use glib::Properties;
    use gtk::{
        glib,
        pango::{self, AttrList},
        prelude::*,
        subclass::{prelude::*, widget::WidgetImpl},
        LevelBar,
    };
    use relm4::view;
    use std::{cell::RefCell, str::FromStr};

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::InfoRowLevel)]
    pub struct InfoRowLevel {
        #[property(get, set)]
        name: RefCell<String>,
        #[property(get, set)]
        value: RefCell<String>,
        #[property(get, set)]
        level_value: RefCell<f64>,

        pub(super) value_label: gtk::Label,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for InfoRowLevel {
        const NAME: &'static str = "InfoRowLevel";
        type Type = super::InfoRowLevel;
        type ParentType = gtk::Box;
    }

    #[glib::derived_properties]
    impl ObjectImpl for InfoRowLevel {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            let value_label = &self.value_label;

            view! {
                #[local_ref]
                obj {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 4,

                    append: name_label = &gtk::Label {
                        set_halign: gtk::Align::Start,
                        set_xalign: 0.0,
                        add_css_class: "caption",
                        add_css_class: "dim-label",
                    },

                    append = &gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 5,

                        #[local_ref]
                        append = value_label {
                            set_attributes: Some(&AttrList::from_str("0 -1 weight bold").unwrap()),
                            set_halign: gtk::Align::Start,
                            set_xalign: 0.0,
                            set_use_markup: true,
                            set_ellipsize: pango::EllipsizeMode::End,
                        },

                        append: level_bar = &LevelBar {
                            set_hexpand: true,
                            set_orientation: gtk::Orientation::Horizontal,
                        },
                    },
                }
            }

            obj.bind_property("name", &name_label, "label")
                .sync_create()
                .build();

            obj.bind_property("value", value_label, "label")
                .sync_create()
                .build();

            obj.bind_property("level-value", &level_bar, "value")
                .sync_create()
                .build();
        }
    }

    impl WidgetImpl for InfoRowLevel {}
    impl BoxImpl for InfoRowLevel {}
}
