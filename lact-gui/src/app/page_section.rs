use gtk::prelude::*;
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

    pub fn append_child(&self, widget: &impl IsA<gtk::Widget>) {
        use glib::subclass::types::ObjectSubclassIsExt;
        self.imp().children_box.append(widget);
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
    use relm4::{css, view, RelmWidgetExt};
    use std::cell::RefCell;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::PageSection)]
    pub struct PageSection {
        section_label: Label,
        pub(super) content_box: gtk::Box,
        pub(super) children_box: gtk::Box,

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

            self.content_box.set_orientation(gtk::Orientation::Vertical);
            let content_box = &self.content_box;

            self.children_box
                .set_orientation(gtk::Orientation::Vertical);
            self.children_box.set_margin_all(10);
            self.children_box.set_spacing(10);
            let children_box = &self.children_box;

            view! {
                #[local_ref]
                obj {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 10,

                    #[local_ref]
                    append = section_label {
                        set_use_markup: true,
                        set_halign: gtk::Align::Start,
                        set_margin_vertical: 5,
                        set_margin_horizontal: 5,
                    },

                    #[local_ref]
                    append = content_box {
                        add_css_class: if cfg!(feature = "adw") { css::CARD } else { css::FRAME },
                        inline_css: if cfg!(feature = "adw") { "" } else { "border-color: @accent_color;" },

                        #[local_ref]
                        append = children_box {}
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
