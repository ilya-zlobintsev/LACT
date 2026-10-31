use super::setting_value::SettingValue;
use relm4::binding::{Binding, U32Binding};

pub struct SelectValue {
    value: U32Binding,
    default_value: Option<u32>,
}

impl SelectValue {
    pub fn new(value: Option<u32>, default_value: Option<u32>, option_count: u32) -> Self {
        Self {
            value: U32Binding::new(value.filter(|value| *value < option_count).unwrap_or(0)),
            default_value,
        }
    }

    pub fn binding(&self) -> &U32Binding {
        &self.value
    }
}

impl SettingValue for SelectValue {
    type Value = u32;

    fn get_value(&self) -> u32 {
        self.value.get()
    }

    fn set_value(&self, value: u32) {
        if self.get_value() != value {
            self.value.set(value);
        }
    }

    fn reset(&self) {
        if let Some(value) = self.default_value {
            self.set_value(value);
        }
    }

    fn serialize(&self) -> Option<u32> {
        let value = self.get_value();
        (Some(value) != self.default_value).then_some(value)
    }
}
