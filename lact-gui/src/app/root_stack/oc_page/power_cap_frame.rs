use gtk::prelude::*;
use gtk::*;

#[derive(Clone)]
pub struct PowerCapFrame {
    pub container: Frame,
    label: Label,
    adjustment: Adjustment,
}

impl PowerCapFrame {
    pub fn new() -> Self {
        let container = Frame::new(None);

        container.set_shadow_type(ShadowType::None);

        container.set_label_widget(Some(&{
            let label = Label::new(None);
            label.set_markup("<span font_desc='11'><b>Power Usage Limit</b></span>");
            label
        }));
        container.set_label_align(0.2, 0.0);

        let root_box = Box::new(Orientation::Horizontal, 0);

        let label = Label::new(None);

        root_box.pack_start(&label, false, true, 5);

        let adjustment = Adjustment::new(0.0, 0.0, 0.0, 1.0, 10.0, 0.0);
        {
            let label = label.clone();
            adjustment.connect_value_changed(move |adj| {
                label.set_markup(&format!("{}/{} W", adj.value().round(), adj.upper()));
            });
        }

        let scale = Scale::new(Orientation::Horizontal, Some(&adjustment));

        scale.set_draw_value(false);

        root_box.pack_start(&scale, true, true, 5);

        container.add(&root_box);

        Self {
            container,
            label,
            adjustment,
        }
    }

    pub fn set_data(&self, power_cap: Option<f64>, power_cap_max: Option<f64>) {
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
    }

    pub fn get_cap(&self) -> Option<i64> {
        // Using match gives a warning that floats shouldn't be used in patterns
        let cap = self.adjustment.value();
        if cap == 0.0 {
            None
        } else {
            Some(cap as i64)
        }
    }

    pub fn connect_cap_changed<F: Fn() + 'static>(&self, f: F) {
        self.adjustment.connect_value_changed(move |_| {
            f();
        });
    }
}
