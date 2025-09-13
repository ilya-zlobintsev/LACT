use gtk::{
    glib::{
        self,
        subclass::types::{IsSubclassable, ObjectSubclass},
        Object,
    },
    subclass::box_::BoxImpl,
};

glib::wrapper! {
    pub struct PageSection(ObjectSubclass<imp::PageSection>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Orientable, gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl PageSection {
    pub fn new(name: &str) -> Self {
        Object::builder().property("name", name).build()
    }
}

unsafe impl<T: ObjectSubclass + BoxImpl> IsSubclassable<T> for PageSection {}

mod imp {
    use glib::Properties;
    use gtk::{
        glib::{self},
        prelude::*,
        subclass::{prelude::*, widget::WidgetImpl},
        Label,
    };
    use relm4::{view, RelmWidgetExt};
    use std::cell::RefCell;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::PageSection)]
    pub struct PageSection {
        section_label: Label,

        #[property(get, set)]
        name: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PageSection {
        const NAME: &'static str = "PageSection";
        type Type = super::PageSection;
        type ParentType = gtk::Box;
    }

    #[glib::derived_properties]
    impl ObjectImpl for PageSection {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            let section_label = &self.section_label;

            view! {
                #[local_ref]
                obj {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 10,
                    set_margin_horizontal: 5,

                    #[local_ref]
                    append = section_label {
                        set_use_markup: true,
                        set_halign: gtk::Align::Start,
                        set_margin_vertical: 5,
                    }
                }
            }

            obj.bind_property("name", &self.section_label, "label")
                .transform_to(|_, value: String| {
                    Some(format!("<span font_desc='13'><b>{value}</b></span>"))
                })
                .build();
        }
    }

    impl WidgetImpl for PageSection {}
    impl BoxImpl for PageSection {}
}
