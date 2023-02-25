mod imp;

use gtk::glib::{self, Object};

glib::wrapper! {
    pub struct FeatureModel(ObjectSubclass<imp::FeatureModel>);
}

impl FeatureModel {
    pub fn new(name: String, supported: bool) -> Self {
        Object::builder()
            .property("name", name)
            .property("supported", supported)
            .build()
    }
}
