use super::setting_value::SettingValue;
use adw::prelude::*;

/// A numeric setting stored in a shared GTK adjustment, with raw/display unit conversion.
pub struct AdjustmentValue {
    value: gtk::Adjustment,
    default_value: Option<f64>,
    value_ratio: f64,
}

impl AdjustmentValue {
    pub fn new(
        value: f64,
        default_value: Option<f64>,
        lower: f64,
        upper: f64,
        step_increment: f64,
        page_increment: f64,
    ) -> Self {
        Self {
            value: gtk::Adjustment::new(value, lower, upper, step_increment, page_increment, 0.0),
            default_value,
            value_ratio: 1.0,
        }
    }

    pub fn adjustment(&self) -> &gtk::Adjustment {
        &self.value
    }

    /// Changes display units, retaining the raw value.
    pub fn set_value_ratio(&mut self, ratio: f64) {
        let value = self.get_value();
        let lower = self.value.lower() / self.value_ratio;
        let upper = self.value.upper() / self.value_ratio;
        self.value_ratio = ratio;
        self.value.set_lower(lower * ratio);
        self.value.set_upper(upper * ratio);
        self.set_value(value);
    }
}

impl SettingValue for AdjustmentValue {
    type Value = f64;

    fn get_value(&self) -> f64 {
        self.value.value() / self.value_ratio
    }

    fn set_value(&self, value: f64) {
        self.value.set_value(value * self.value_ratio);
    }

    fn reset(&self) {
        if let Some(value) = self.default_value {
            self.set_value(value);
        }
    }

    fn serialize(&self) -> Option<f64> {
        let value = self.get_value();
        (Some(value) != self.default_value).then_some(value)
    }
}
