use gtk::glib::{self, Object};

glib::wrapper! {
    pub struct InfoRow(ObjectSubclass<imp::InfoRow>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Orientable, gtk::Accessible, gtk::Buildable;
}

impl InfoRow {
    pub fn new(name: &str, value: &str) -> Self {
        Object::builder()
            .property("name", name)
            .property("value", value)
            .build()
    }
}

mod imp {
    use glib::Properties;
    use gtk::{
        glib::{self, subclass::InitializingObject},
        pango::AttrList,
        prelude::*,
        subclass::{
            prelude::*,
            widget::{CompositeTemplateClass, WidgetImpl},
        },
        CompositeTemplate, Label, TemplateChild,
    };
    use std::{cell::RefCell, str::FromStr};

    #[derive(CompositeTemplate, Default, Properties)]
    #[properties(wrapper_type = super::InfoRow)]
    #[template(file = "ui/info_row.blp")]
    pub struct InfoRow {
        #[property(get, set)]
        name: RefCell<String>,
        #[property(get, set)]
        value: RefCell<String>,

        #[template_child]
        value_label: TemplateChild<Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for InfoRow {
        const NAME: &'static str = "InfoRow";
        type Type = super::InfoRow;
        type ParentType = gtk::Box;

        fn class_init(class: &mut Self::Class) {
            class.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for InfoRow {
        fn constructed(&self) {
            self.parent_constructed();

            let attr_list = AttrList::from_str("0 -1 weight bold").unwrap();
            self.value_label.set_attributes(Some(&attr_list));
        }
    }

    impl WidgetImpl for InfoRow {}
    impl BoxImpl for InfoRow {}
}
