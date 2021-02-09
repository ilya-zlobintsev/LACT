use gtk::*;

#[derive(Clone)]
pub struct ApplyRevealer {
    pub container: Revealer,
    apply_button: Button,
}

impl ApplyRevealer {
    pub fn new() -> Self {
        let container = Revealer::new();

        container.set_transition_duration(150);

        let apply_button = Button::new();

        apply_button.set_label("Apply");

        container.add(&apply_button);

        Self {
            container,
            apply_button,
        }
    }

    pub fn show(&self) {
        self.container.set_reveal_child(true);
    }

    pub fn hide(&self) {
        self.container.set_reveal_child(false);
    }

    pub fn connect_apply_button_clicked<F: Fn() + 'static>(&self, f: F) {
        let apply_revealer = self.container.clone();

        self.apply_button.connect_clicked(move |_| {
            f();
        });
    }
}
