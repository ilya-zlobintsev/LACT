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

    pub fn new_selectable(name: &str, value: &str) -> Self {
        Object::builder()
            .property("name", name)
            .property("value", value)
            .property("selectable", true)
            .build()
    }
}

impl Default for InfoRow {
    fn default() -> Self {
        Object::builder().build()
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
        CompositeTemplate, Label, MenuButton, TemplateChild,
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
        #[property(get, set)]
        selectable: RefCell<bool>,
        #[property(get, set)]
        info_text: RefCell<String>,

        #[template_child]
        value_label: TemplateChild<Label>,
        #[template_child]
        info_menubutton: TemplateChild<MenuButton>,
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

            let obj = self.obj();
            obj.bind_property("info-text", &self.info_menubutton.get(), "visible")
                .transform_to(|_, text: String| Some(!text.is_empty()))
                .sync_create()
                .build();

            obj.bind_property("value", &self.info_menubutton.get(), "visible")
                .transform_to(|_, text: String| {
                    if text.starts_with("Unknown ") {
                        Some(false)
                    } else {
                        None
                    }
                })
                .sync_create()
                .build();
        }
    }

    impl WidgetImpl for InfoRow {}
    impl BoxImpl for InfoRow {}
}
