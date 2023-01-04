use gio::subclass::prelude::*;
use gtk::{
    gio,
    glib::{self, ParamSpec, ParamSpecBoolean, ParamSpecString},
    prelude::*,
};
use once_cell::sync::Lazy;
use std::cell::{Cell, RefCell};

#[derive(Debug, Default)]
pub struct FeatureModel {
    pub name: RefCell<String>,
    pub supported: Cell<bool>,
}

#[glib::object_subclass]
impl ObjectSubclass for FeatureModel {
    const NAME: &'static str = "VulkanFeatureModel";
    type Type = super::FeatureModel;
}

impl ObjectImpl for FeatureModel {
    fn properties() -> &'static [glib::ParamSpec] {
        static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
            vec![
                ParamSpecString::builder("name").build(),
                ParamSpecBoolean::builder("supported").build(),
            ]
        });
        PROPERTIES.as_ref()
    }

    fn set_property(&self, _id: usize, value: &glib::Value, pspec: &ParamSpec) {
        match pspec.name() {
            "name" => {
                let name = value.get().expect("Name needs to be a string");
                self.name.replace(name);
            }
            "supported" => {
                let supported = value.get().expect("Supported needs to be a bool");
                self.supported.replace(supported);
            }
            _ => unimplemented!(),
        }
    }

    fn property(&self, _id: usize, pspec: &ParamSpec) -> glib::Value {
        match pspec.name() {
            "name" => self.name.borrow().to_value(),
            "supported" => self.supported.get().to_value(),
            _ => unimplemented!(),
        }
    }
}
