use gtk::prelude::*;
use gtk::{
    glib::{
        self, Object,
        subclass::types::{IsSubclassable, ObjectSubclass},
    },
    subclass::box_::BoxImpl,
};

glib::wrapper! {
    pub struct PageSectionExpander(ObjectSubclass<imp::PageSectionExpander>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Orientable, gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl PageSectionExpander {
    pub fn new(name: &str) -> Self {
        Object::builder().property("name", name).build()
    }

    pub fn append_header(&self, widget: &impl IsA<gtk::Widget>) {
        use glib::subclass::types::ObjectSubclassIsExt;
        self.imp().header_box.append(widget);
    }

    pub fn append_expandable(&self, widget: &impl IsA<gtk::Widget>) {
        use glib::subclass::types::ObjectSubclassIsExt;
        self.imp().children_box.append(widget);
    }
}

unsafe impl<T: ObjectSubclass + BoxImpl> IsSubclassable<T> for PageSectionExpander {}

mod imp {
    use glib::Properties;
    use gtk::{
        Label,
        glib::{self},
        prelude::*,
        subclass::{prelude::*, widget::WidgetImpl},
    };
    use relm4::{RelmWidgetExt, view};
    use std::cell::RefCell;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::PageSectionExpander)]
    pub struct PageSectionExpander {
        section_label: Label,
        pub(super) header_box: gtk::Box,
        pub(super) content_box: gtk::Box,
        pub(super) children_box: gtk::Box,
        pub(super) expander: gtk::Expander,

        #[property(get, set)]
        name: RefCell<String>,

        #[property(get, set)]
        expanded: std::cell::Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PageSectionExpander {
        const NAME: &'static str = "PageSectionExpander";
        type Type = super::PageSectionExpander;
        type ParentType = gtk::Box;
    }

    #[glib::derived_properties]
    impl ObjectImpl for PageSectionExpander {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            let section_label = &self.section_label;
            let header_box = &self.header_box;
            let content_box = &self.content_box;
            let children_box = &self.children_box;
            let expander = &self.expander;

            view! {
                #[local_ref]
                obj {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 10,

                    #[local_ref]
                    append = expander {
                        set_child: Some(content_box),
                        set_label_widget : Some(header_box),
                    },
                },

                #[local_ref]
                header_box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 10,

                    #[local_ref]
                    append = section_label {
                        set_use_markup: true,
                        set_halign: gtk::Align::Start,
                        set_margin_vertical: 5,
                    },
                },

                #[local_ref]
                content_box {
                    set_orientation: gtk::Orientation::Vertical,

                    #[local_ref]
                    append = children_box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 10,
                        set_margin_horizontal: 15,
                    },
                },
            }

            obj.bind_property("name", &self.section_label, "label")
                .transform_to(|_, value: String| {
                    Some(format!("<span font_desc='13'><b>{value}</b></span>"))
                })
                .build();

            obj.bind_property("expanded", expander, "expanded")
                .bidirectional()
                .build();
        }
    }

    impl WidgetImpl for PageSectionExpander {}
    impl BoxImpl for PageSectionExpander {}
}
