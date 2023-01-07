use super::{oc_adjustment, section_box};
use gtk::prelude::*;
use gtk::*;
use std::{cell::Cell, rc::Rc};

#[derive(Clone)]
pub struct PowerCapFrame {
    pub container: Box,
    default_cap: Rc<Cell<Option<f64>>>,
    adjustment: Adjustment,
}

impl PowerCapFrame {
    pub fn new() -> Self {
        let container = section_box("Power Usage Limit", 5, 5);
        let default_cap = Rc::new(Cell::new(None));
        let (root_box, adjustment) = oc_adjustment(Some(default_cap.clone()), "W");
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
