use gtk::prelude::*;
use gtk::*;

#[derive(Clone)]
pub struct ApplyRevealer {
    pub container: Revealer,
    apply_button: Button,
    reset_button: Button,
}

impl ApplyRevealer {
    pub fn new() -> Self {
        let container = Revealer::builder().transition_duration(150).build();
        let vbox = Box::new(Orientation::Horizontal, 5);

        let apply_button = Button::builder().label("Apply").hexpand(true).build();
        let reset_button = Button::builder().label("Reset").build();

        vbox.append(&apply_button);
        vbox.append(&reset_button);

        container.set_child(Some(&vbox));

        Self {
            container,
            apply_button,
            reset_button,
        }
    }

    pub fn show(&self) {
        self.container.set_reveal_child(true);
    }

    pub fn hide(&self) {
        self.container.set_reveal_child(false);
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
