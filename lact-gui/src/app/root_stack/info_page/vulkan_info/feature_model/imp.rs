use gio::subclass::prelude::*;
use gtk::{
    gio,
    glib::{self, Properties},
    prelude::*,
};
use std::cell::{Cell, RefCell};

#[derive(Debug, Default, Properties)]
#[properties(wrapper_type = super::FeatureModel)]
pub struct FeatureModel {
    #[property(set, get)]
    pub name: RefCell<String>,
    #[property(set, get)]
    pub supported: Cell<bool>,
}

#[glib::object_subclass]
impl ObjectSubclass for FeatureModel {
    const NAME: &'static str = "VulkanFeatureModel";
    type Type = super::FeatureModel;
}

#[glib::derived_properties]
impl ObjectImpl for FeatureModel {}
