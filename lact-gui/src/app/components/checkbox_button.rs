use adw::prelude::*;

#[relm4::widget_template(pub)]
impl relm4::WidgetTemplate for CheckboxButton {
    view! {
        gtk::ToggleButton {
            add_css_class: "checkbox-button",
            #[chain(sync_create().build())]
            bind_property: ("active", &_checkbox, "active"),

            #[wrap(Some)]
            set_child = &gtk::Box {
                set_spacing: 6,

                #[name = "_checkbox"]
                gtk::CheckButton {
                    set_can_target: false,
                    set_focusable: false,
                },

                #[name = "label"]
                gtk::Label {},
            },
        }
    }
}

impl CheckboxButton {
    pub fn set_label(&self, label: &str) {
        self.label.set_label(label);
    }
}
