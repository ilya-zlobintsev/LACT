use glib::clone;
use gtk::{
    glib, prelude::*, Adjustment, Box, Label, MenuButton, Orientation, Popover, Scale, SpinButton,
};

#[derive(Clone)]
pub struct PointAdjustment {
    pub temperature: Adjustment,
    pub ratio: Adjustment,
}

impl PointAdjustment {
    pub fn new(parent: &Box, ratio: f32, temperature: i32) -> Self {
        let container = Box::new(Orientation::Vertical, 5);

        let ratio_adjustment = Adjustment::new(ratio.into(), 0.0, 1.0, 0.01, 0.05, 0.00);
        let scale = Scale::builder()
            .orientation(Orientation::Vertical)
            .adjustment(&ratio_adjustment)
            .hexpand(true)
            .vexpand(true)
            .inverted(true)
            .build();
        container.append(&scale);

        let temperature_adjustment = Adjustment::new(temperature.into(), 0.0, 100.0, 1.0, 1.0, 0.0);
        let temperature_selector = SpinButton::new(Some(&temperature_adjustment), 1.0, 0);

        // Using the built-in MenuButton label function creates an empty icon
        let temperature_label = Label::new(Some(&temperature.to_string()));

        temperature_adjustment.connect_value_changed(
            clone!(@strong temperature_label => move |temperature_adjustment| {
                let temperature = temperature_adjustment.value();
                temperature_label.set_text(&temperature.to_string());
            }),
        );

        let popover = Popover::builder().child(&temperature_selector).build();
        let temperature_button = MenuButton::builder()
            .popover(&popover)
            .child(&temperature_label)
            .build();

        container.append(&temperature_button);

        parent.append(&container);

        Self {
            temperature: temperature_adjustment,
            ratio: ratio_adjustment,
        }
    }
}
