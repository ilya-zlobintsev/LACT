use gtk::glib::{self, Object};

glib::wrapper! {
    pub struct PageSection(ObjectSubclass<imp::PageSection>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Orientable, gtk::Accessible, gtk::Buildable;
}

impl PageSection {
    pub fn new(name: &str) -> Self {
        Object::builder().property("name", name).build()
    }
}

mod imp {
    use glib::Properties;
    use gtk::{
        glib::{self, subclass::InitializingObject},
        prelude::*,
        subclass::{
            prelude::*,
            widget::{CompositeTemplateClass, WidgetImpl},
        },
        CompositeTemplate, Label, TemplateChild,
    };
    use std::cell::RefCell;

    #[derive(CompositeTemplate, Default, Properties)]
    #[properties(wrapper_type = super::PageSection)]
    #[template(file = "ui/page_section.blp")]
    pub struct PageSection {
        #[template_child]
        section_label: TemplateChild<Label>,

        #[property(get, set)]
        name: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PageSection {
        const NAME: &'static str = "PageSection";
        type Type = super::PageSection;
        type ParentType = gtk::Box;

        fn class_init(class: &mut Self::Class) {
            class.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for PageSection {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            obj.bind_property("name", &self.section_label.get(), "label")
                .transform_to(|_, value: String| {
                    Some(format!("<span font_desc='13'><b>{value}</b></span>"))
                })
                .build();
        }
    }

    impl WidgetImpl for PageSection {}
    impl BoxImpl for PageSection {}
}
