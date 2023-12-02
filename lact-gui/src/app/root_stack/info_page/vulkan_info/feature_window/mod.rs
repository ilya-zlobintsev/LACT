pub mod feature;
mod row;

use glib::Object;
use gtk::{gio, glib};

#[cfg(feature = "libadwaita")]
glib::wrapper! {
    pub struct VulkanFeaturesWindow(ObjectSubclass<imp::VulkanFeaturesWindow>)
        @extends gtk::Box, gtk::Widget, gtk::Window, libadwaita::Window,
        @implements gtk::Orientable, gtk::Accessible, gtk::Buildable;
}

#[cfg(not(feature = "libadwaita"))]
glib::wrapper! {
    pub struct VulkanFeaturesWindow(ObjectSubclass<imp::VulkanFeaturesWindow>)
        @extends gtk::Box, gtk::Widget, gtk::Window,
        @implements gtk::Orientable, gtk::Accessible, gtk::Buildable;
}

impl VulkanFeaturesWindow {
    pub fn new(title: &str, model: gio::ListModel) -> Self {
        Object::builder()
            .property("title", title)
            .property("model", model)
            .build()
    }
}

mod imp {
    use super::{feature::VulkanFeature, row::VulkanFeatureRow};
    use glib::Properties;
    use gtk::{
        gio,
        glib::{self, clone, subclass::InitializingObject},
        prelude::*,
        subclass::{
            prelude::*,
            widget::{CompositeTemplateClass, WidgetImpl},
        },
        CompositeTemplate, Expression, FilterListModel, PropertyExpression, SearchEntry,
        SignalListItemFactory, StringFilter, TemplateChild,
    };
    use std::cell::RefCell;

    #[cfg(feature = "libadwaita")]
    use libadwaita::subclass::window::AdwWindowImpl;

    #[derive(CompositeTemplate, Properties, Default)]
    #[properties(wrapper_type = super::VulkanFeaturesWindow)]
    #[cfg_attr(feature = "libadwaita", template(file = "ui/vulkan_features_window.blp"))]
    #[cfg_attr(not(feature = "libadwaita"), template(file = "ui/vulkan_features_window_gtk.blp"))]
    pub struct VulkanFeaturesWindow {
        #[property(get, set)]
        model: RefCell<Option<gio::ListModel>>,
        #[template_child]
        features_factory: TemplateChild<SignalListItemFactory>,

        #[template_child]
        filter_model: TemplateChild<FilterListModel>,

        #[template_child]
        search_filter: TemplateChild<StringFilter>,
        #[template_child]
        search_entry: TemplateChild<SearchEntry>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for VulkanFeaturesWindow {
        const NAME: &'static str = "VulkanFeaturesWindow";
        type Type = super::VulkanFeaturesWindow;

        #[cfg(feature = "libadwaita")]
        type ParentType = libadwaita::Window;

        #[cfg(not(feature = "libadwaita"))]
        type ParentType = gtk::Window;

        fn class_init(class: &mut Self::Class) {
            class.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for VulkanFeaturesWindow {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            obj.bind_property("model", &self.filter_model.get(), "model")
                .sync_create()
                .build();

            let expression =
                PropertyExpression::new(VulkanFeature::static_type(), Expression::NONE, "name");
            self.search_filter.set_expression(Some(&expression));

            self.search_entry.connect_search_changed(
                clone!(@strong self.search_filter as filter => move |entry| {
                    if entry.text().is_empty() {
                        filter.set_search(None);
                    } else {
                        filter.set_search(Some(entry.text().as_str()));
                    }
                }),
            );

            self.features_factory.connect_setup(|_, list_item| {
                let feature = VulkanFeature::default();
                let row = VulkanFeatureRow::new(feature);
                list_item.set_child(Some(&row));
            });

            self.features_factory.connect_bind(|_, list_item| {
                let feature = list_item
                    .item()
                    .and_downcast::<VulkanFeature>()
                    .expect("The item has to be a VulkanFeature");

                let row = list_item
                    .child()
                    .and_downcast::<VulkanFeatureRow>()
                    .expect("The child has to be a VulkanFeatureRow");
                row.set_feature(feature);
            });
        }
    }

    impl WidgetImpl for VulkanFeaturesWindow {}
    impl WindowImpl for VulkanFeaturesWindow {}

    #[cfg(feature = "libadwaita")]
    impl AdwWindowImpl for VulkanFeaturesWindow {}

    impl ApplicationWindowImpl for VulkanFeaturesWindow {}
}
