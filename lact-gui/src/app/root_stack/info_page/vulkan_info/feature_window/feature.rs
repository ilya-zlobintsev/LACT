use gtk::glib::{self, Object};

glib::wrapper! {
    pub struct VulkanFeature(ObjectSubclass<imp::VulkanFeature>);
}

impl VulkanFeature {
    pub fn new(name: String, supported: bool) -> Self {
        Object::builder()
            .property("name", name)
            .property("supported", supported)
            .build()
    }
}

impl Default for VulkanFeature {
    fn default() -> Self {
        Self::new(String::new(), false)
    }
}

mod imp {
    use gio::subclass::prelude::*;
    use gtk::{
        gio,
        glib::{self, Properties},
        prelude::*,
    };
    use std::cell::{Cell, RefCell};

    #[derive(Debug, Default, Properties)]
    #[properties(wrapper_type = super::VulkanFeature)]
    pub struct VulkanFeature {
        #[property(set, get)]
        pub name: RefCell<String>,
        #[property(set, get)]
        pub supported: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for VulkanFeature {
        const NAME: &'static str = "VulkanFeature";
        type Type = super::VulkanFeature;
    }

    #[glib::derived_properties]
    impl ObjectImpl for VulkanFeature {}
}
