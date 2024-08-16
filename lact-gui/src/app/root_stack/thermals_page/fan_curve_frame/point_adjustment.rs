use glib::clone;
use gtk::{
    glib, prelude::*, Adjustment, Box, Grid, Label, MenuButton, Orientation, Popover, Scale,
    SpinButton,
};

#[derive(Clone)]
pub struct PointAdjustment {
    pub temperature: Adjustment,
    pub ratio: Adjustment,
}

impl PointAdjustment {
    pub fn new(parent: &Box, ratio: f32, temperature: i32) -> Self {
        let container = Box::new(Orientation::Vertical, 5);
        container.set_margin_top(10);

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
        let ratio_selector = SpinButton::new(Some(&ratio_adjustment), 0.05, 2);

        temperature_selector.connect_input(|spin| {
            let text = spin.text();
            let temp = text.trim_end_matches("°C");
            Some(Ok(temp.parse::<f64>().unwrap_or_else(|_| spin.value())))
        });
        temperature_selector.connect_output(|spin| {
            let text = format!("{}°C", spin.value_as_int());
            spin.set_text(&text);
            glib::Propagation::Stop
        });

        ratio_selector.connect_input(|spin| {
            let text = spin.text();
            let percentage = text.trim_end_matches('%');
            Some(Ok(percentage
                .parse::<f64>()
                .map(|value| value / 100.0)
                .unwrap_or_else(|_| spin.value())))
        });
        ratio_selector.connect_output(|spin| {
            let value = spin.value();
            let ratio = (value * 100.0).round();
            spin.set_text(&format!("{ratio}%"));
            glib::Propagation::Stop
        });

        let popover_menu = Grid::builder()
            .column_spacing(5)
            .row_spacing(5)
            .margin_start(5)
            .margin_end(5)
            .margin_top(5)
            .margin_bottom(5)
            .build();
        popover_menu.attach(&Label::new(Some("Speed:")), 0, 0, 1, 1);
        popover_menu.attach(&ratio_selector, 1, 0, 1, 1);
        popover_menu.attach(&Label::new(Some("Temperature:")), 0, 1, 1, 1);
        popover_menu.attach(&temperature_selector, 1, 1, 1, 1);

        // Using the built-in MenuButton label function creates an empty icon
        let text = format!("<b>{}%</b> at {temperature}°C", (ratio * 100.0).round());
        let temperature_label = Label::builder().label(text).use_markup(true).build();

        temperature_adjustment.connect_value_changed(clone!(
            #[strong]
            temperature_label,
            #[strong]
            ratio_adjustment,
            move |temperature_adjustment| {
                let temperature = temperature_adjustment.value();
                let ratio = (ratio_adjustment.value() * 100.0).round();
                let text = format!("<b>{ratio}%</b> at {temperature}°C");
                temperature_label.set_markup(&text);
            }
        ));

        ratio_adjustment.connect_value_changed(clone!(
            #[strong]
            temperature_adjustment,
            move |_| {
                temperature_adjustment.emit_by_name::<()>("value-changed", &[]);
            }
        ));

        let popover = Popover::builder().child(&popover_menu).build();
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
