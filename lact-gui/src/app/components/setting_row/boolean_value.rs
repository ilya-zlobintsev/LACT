use super::setting_value::SettingValue;
use relm4::binding::{Binding, BoolBinding};

pub struct BooleanValue {
    value: BoolBinding,
    default_value: Option<bool>,
}

impl BooleanValue {
    pub fn new(value: bool, default_value: Option<bool>) -> Self {
        Self {
            value: BoolBinding::new(value),
            default_value,
        }
    }

    pub fn binding(&self) -> &BoolBinding {
        &self.value
    }
}

impl SettingValue for BooleanValue {
    type Value = bool;

    fn get_value(&self) -> bool {
        self.value.get()
    }

    fn set_value(&self, value: bool) {
        if self.get_value() != value {
            self.value.set(value);
        }
    }

    fn reset(&self) {
        if let Some(value) = self.default_value {
            self.set_value(value);
        }
    }

    fn serialize(&self) -> Option<bool> {
        let value = self.get_value();
        (Some(value) != self.default_value).then_some(value)
    }
}
