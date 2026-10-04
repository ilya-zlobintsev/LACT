use adw::prelude::*;

#[relm4::widget_template(pub)]
impl relm4::WidgetTemplate for CheckboxButton {
    view! {
        gtk::CheckButton::builder()
            .css_name("button")
            .build() {
            add_css_class: "toggle",
            add_css_class: "checkbox-button",
        }
    }
}
