use glib::clone;
use gtk::{
    glib,
    prelude::ScaleExt,
    traits::{AdjustmentExt, BoxExt},
    Adjustment, Box, Label, MenuButton, Orientation, Popover, Scale, SpinButton,
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
            .tooltip_text("Fan speed")
            .digits(3)
            .draw_value(true)
            .build();
        scale.set_format_value_func(|_, v| format!("{:.1}%", v * 100.0));
        container.append(&scale);

        let temperature_adjustment = Adjustment::new(temperature.into(), 0.0, 100.0, 1.0, 1.0, 0.0);
        let temperature_selector = SpinButton::new(Some(&temperature_adjustment), 1.0, 0);

        // Using the built-in MenuButton label function creates an empty icon
        let temperature_label = Label::builder()
            .label(&format!("{}°C", temperature))
            .margin_start(6)
            .margin_end(6)
            .build();

        temperature_adjustment.connect_value_changed(
            clone!(@strong temperature_label => move |temperature_adjustment| {
                let temperature = temperature_adjustment.value();
                temperature_label.set_text(&format!("{}°C", temperature));
            }),
        );

        let popover = Popover::builder().child(&temperature_selector).build();
        let temperature_button = MenuButton::builder()
            .hexpand(false)
            .halign(gtk::Align::Center)
            .css_classes(["circular"])
            .tooltip_text("Temperature")
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
