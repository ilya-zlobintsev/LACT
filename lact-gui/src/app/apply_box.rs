use gtk::prelude::*;
use gtk::*;

#[derive(Clone)]
pub struct ApplyBox {
    pub container: Box,
    apply_button: Button,
    reset_button: Button,
}

impl ApplyBox {
    pub fn new() -> Self {
        let container = Box::builder()
            .orientation(Orientation::Horizontal)
            .css_classes(["linked"])
            .visible(false)
            .build();

        let apply_button = Button::builder()
            .css_classes(["suggested-action"])
            .label("Apply")
            .build();
        let reset_button = Button::builder()
            .icon_name("view-refresh-symbolic")
            .tooltip_text("Reset")
            .build();

        container.append(&apply_button);
        container.append(&reset_button);

        Self {
            container,
            apply_button,
            reset_button,
        }
    }

    pub fn show(&self) {
        self.container.set_visible(true);
    }

    pub fn hide(&self) {
        self.container.set_visible(false);
    }

    pub fn connect_apply_button_clicked<F: Fn() + 'static>(&self, f: F) {
        self.apply_button.connect_clicked(move |_| {
            f();
        });
    }

    pub fn connect_reset_button_clicked<F: Fn() + 'static>(&self, f: F) {
        self.reset_button.connect_clicked(move |_| {
            f();
        });
    }
}
