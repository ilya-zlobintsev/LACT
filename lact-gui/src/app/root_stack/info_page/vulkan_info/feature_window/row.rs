use super::feature::VulkanFeature;
use glib::Object;
use gtk::glib;

glib::wrapper! {
    pub struct VulkanFeatureRow(ObjectSubclass<imp::VulkanFeatureRow>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Orientable, gtk::Accessible, gtk::Buildable;
}

impl VulkanFeatureRow {
    pub fn new(feature: VulkanFeature) -> Self {
        Object::builder().property("feature", feature).build()
    }
}

mod imp {
    use crate::app::root_stack::info_page::vulkan_info::feature_window::feature::VulkanFeature;
    use glib::Properties;
    use gtk::{
        glib::{self, subclass::InitializingObject},
        prelude::*,
        subclass::{
            prelude::*,
            widget::{CompositeTemplateClass, WidgetImpl},
        },
        CompositeTemplate, Image, Label, TemplateChild,
    };
    use std::cell::RefCell;

    #[derive(CompositeTemplate, Default, Properties)]
    #[properties(wrapper_type = super::VulkanFeatureRow)]
    #[template(file = "ui/vulkan_feature_row.blp")]
    pub struct VulkanFeatureRow {
        #[template_child]
        name_label: TemplateChild<Label>,
        #[template_child]
        available_image: TemplateChild<Image>,

        #[property(get, set)]
        feature: RefCell<VulkanFeature>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for VulkanFeatureRow {
        const NAME: &'static str = "VulkanFeatureRow";
        type Type = super::VulkanFeatureRow;
        type ParentType = gtk::Box;

        fn class_init(class: &mut Self::Class) {
            class.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for VulkanFeatureRow {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            obj.bind_property("feature", &self.name_label.get(), "label")
                .transform_to(|_, feature: VulkanFeature| Some(feature.name()))
                .sync_create()
                .build();

            obj.bind_property("feature", &self.available_image.get(), "icon-name")
                .transform_to(|_, feature: VulkanFeature| {
                    if feature.supported() {
                        Some("emblem-ok-symbolic")
                    } else {
                        Some("action-unavailable-symbolic")
                    }
                })
                .sync_create()
                .build();
        }
    }

    impl WidgetImpl for VulkanFeatureRow {}
    impl BoxImpl for VulkanFeatureRow {}
}
