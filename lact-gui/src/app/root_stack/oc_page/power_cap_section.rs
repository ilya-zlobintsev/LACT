use crate::app::page_section::PageSection;
use gtk::glib::{self, subclass::types::ObjectSubclassIsExt, Object};

glib::wrapper! {
    pub struct PowerCapSection(ObjectSubclass<imp::PowerCapSection>)
        @extends PageSection, gtk::Box, gtk::Widget,
        @implements gtk::Orientable, gtk::Accessible, gtk::Buildable;
}

impl PowerCapSection {
    pub fn new() -> Self {
        Object::builder().build()
    }

    pub fn get_user_cap(&self) -> Option<f64> {
        let imp = self.imp();
        imp.adjustment.get_changed_value(true)
    }

    pub fn set_initial_value(&self, value: f64) {
        self.imp().adjustment.set_initial_value(value);
    }
}

impl Default for PowerCapSection {
    fn default() -> Self {
        Self::new()
    }
}

mod imp {
    use crate::app::{page_section::PageSection, root_stack::oc_adjustment::OcAdjustment};
    use gtk::{
        glib::{self, clone, subclass::InitializingObject, Properties, StaticTypeExt},
        prelude::{ButtonExt, ObjectExt},
        subclass::{
            prelude::*,
            widget::{CompositeTemplateClass, WidgetImpl},
        },
        Button, CompositeTemplate,
    };
    use std::cell::RefCell;

    #[derive(CompositeTemplate, Default, Properties)]
    #[properties(wrapper_type = super::PowerCapSection)]
    #[template(file = "ui/oc_page/power_cap_section.blp")]
    pub struct PowerCapSection {
        #[property(get, set)]
        pub current_value: RefCell<f64>,
        #[property(get, set)]
        pub max_value: RefCell<f64>,
        #[property(get, set)]
        pub min_value: RefCell<f64>,
        #[property(get, set)]
        pub default_value: RefCell<f64>,
        #[property(get, set)]
        pub value_text: RefCell<String>,

        #[template_child]
        pub adjustment: TemplateChild<OcAdjustment>,
        #[template_child]
        pub reset_button: TemplateChild<Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PowerCapSection {
        const NAME: &'static str = "PowerCapSection";
        type Type = super::PowerCapSection;
        type ParentType = PageSection;

        fn class_init(class: &mut Self::Class) {
            OcAdjustment::ensure_type();
            class.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for PowerCapSection {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            obj.connect_current_value_notify(clone!(@strong obj => move |section| {
                let text = format!("{}/{} W", section.current_value(), section.max_value());
                section.set_value_text(text);
            }));

            self.reset_button
                .connect_clicked(clone!(@strong obj => move |_| {
                    obj.set_current_value(obj.default_value());
                }));
        }
    }

    impl WidgetImpl for PowerCapSection {}
    impl BoxImpl for PowerCapSection {}
}
