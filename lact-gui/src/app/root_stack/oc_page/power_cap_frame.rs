use crate::app::root_stack::section_box;
use gtk::*;
use gtk::{glib::clone, prelude::*};
use std::{cell::Cell, rc::Rc};
use tracing::error;

#[derive(Clone)]
pub struct PowerCapFrame {
    pub container: Box,
    default_cap: Rc<Cell<Option<f64>>>,
    adjustment: Adjustment,
}

impl PowerCapFrame {
    pub fn new() -> Self {
        let container = section_box("Power Usage Limit");
        let default_cap = Rc::new(Cell::new(None));

        let value_suffix = "W";
        let root_box = Box::new(Orientation::Horizontal, 0);

        let label = Label::new(None);
        root_box.append(&label);

        let adjustment = Adjustment::new(0.0, 0.0, 0.0, 1.0, 10.0, 0.0);

        adjustment.connect_value_changed(clone!(@strong label => move |adj| {
            let text = format!("{}/{} {}", adj.value().round(), adj.upper(), value_suffix);
            label.set_label(&text);
        }));

        let scale = Scale::builder()
            .orientation(Orientation::Horizontal)
            .adjustment(&adjustment)
            .hexpand(true)
            .round_digits(0)
            .margin_start(5)
            .margin_end(5)
            .build();

        scale.set_draw_value(false);

        root_box.append(&scale);

        let reset_button = Button::with_label("Default");
        reset_button.connect_clicked(clone!(@strong adjustment, @strong default_cap => move |_| {
            if let Some(cap) = default_cap.get() {
                adjustment.set_value(cap);
            } else {
                error!("Could not set default cap, value not provided");
            }
        }));
        root_box.append(&reset_button);

        container.append(&root_box);

        Self {
            container,
            adjustment,
            default_cap,
        }
    }

    pub fn set_data(
        &self,
        power_cap: Option<f64>,
        power_cap_max: Option<f64>,
        power_cap_default: Option<f64>,
    ) {
        if let Some(power_cap_max) = power_cap_max {
            self.adjustment.set_upper(power_cap_max);
        } else {
            self.container.set_visible(false);
        }

        if let Some(power_cap) = power_cap {
            self.adjustment.set_value(power_cap);
        } else {
            self.container.set_visible(false);
        }

        self.default_cap.set(power_cap_default);
    }

    pub fn get_cap(&self) -> Option<f64> {
        // Using match gives a warning that floats shouldn't be used in patterns
        let cap = self.adjustment.value();
        if cap == 0.0 {
            None
        } else {
            Some(cap)
        }
    }

    pub fn connect_cap_changed<F: Fn() + 'static>(&self, f: F) {
        self.adjustment.connect_value_changed(move |_| {
            f();
        });
    }
}
