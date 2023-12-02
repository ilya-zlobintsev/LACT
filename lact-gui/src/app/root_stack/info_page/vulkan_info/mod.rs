mod feature_window;

use self::feature_window::VulkanFeaturesWindow;
use crate::app::root_stack::info_page::vulkan_info::feature_window::feature::VulkanFeature;
use crate::app::root_stack::{action_row, LabelRow};
use glib::clone;
use gtk::prelude::*;
use gtk::*;
use lact_client::schema::VulkanInfo;
use tracing::trace;

#[cfg(feature = "adw")]
use adw::prelude::ActionRowExt;

#[derive(Debug, Clone)]
pub struct VulkanInfoFrame {
    pub container: ListBox,
    device_name_row: LabelRow,
    version_row: LabelRow,
    driver_name_row: LabelRow,
    driver_version_row: LabelRow,
    features_model: gio::ListStore,
    extensions_model: gio::ListStore,
}

impl VulkanInfoFrame {
    pub fn new() -> Self {
        let container = ListBox::builder()
            .css_classes(["boxed-list"])
            .selection_mode(SelectionMode::None)
            .build();

        let features_model = gio::ListStore::new::<VulkanFeature>();
        let extensions_model = gio::ListStore::new::<VulkanFeature>();

        let device_name_row = LabelRow::new("Device name");
        let version_row = LabelRow::new("Vulkan version");
        let driver_name_row = LabelRow::new("Driver name");
        let driver_version_row = LabelRow::new("Driver version");

        container.append(&device_name_row.container);
        container.append(&version_row.container);
        container.append(&driver_name_row.container);
        container.append(&driver_version_row.container);

        #[cfg(feature = "adw")]
        {
            let features_row = adw::ActionRow::builder()
                .activatable(true)
                .title("Features")
                .build();
            features_row.add_suffix(&Image::from_icon_name("go-next-symbolic"));
            features_row.connect_activated(clone!(@strong features_model => move |_| {
                show_features_window("Vulkan features", features_model.clone());
            }));
            container.append(&features_row);

            let extensions_row = adw::ActionRow::builder()
                .activatable(true)
                .title("Extensions")
                .build();
            extensions_row.add_suffix(&Image::from_icon_name("go-next-symbolic"));
            extensions_row.connect_activated(clone!(@strong extensions_model => move |_| {
                show_features_window("Vulkan extensions", extensions_model.clone());
            }));
            container.append(&extensions_row);
        }

        #[cfg(not(feature = "adw"))]
        {
            let features_btn = Button::builder().label("View").build();
            features_btn.connect_clicked(clone!(@strong features_model => move |_| {
                show_features_window("Vulkan features", features_model.clone());
            }));
            let features_row = action_row("Features", None, &[&features_btn], None);
            container.append(&features_row);

            let extensions_btn = Button::builder().label("View").build();
            extensions_btn.connect_clicked(clone!(@strong extensions_model => move |_| {
                show_features_window("Vulkan extensions", extensions_model.clone());
            }));
            let extensions_row = action_row("Extensions", None, &[&extensions_btn], None);
            container.append(&extensions_row);
        }

        Self {
            container,
            device_name_row,
            version_row,
            driver_name_row,
            driver_version_row,
            features_model,
            extensions_model,
        }
    }

    pub fn set_info(&self, vulkan_info: &VulkanInfo) {
        trace!("setting vulkan info: {:?}", vulkan_info);

        self.device_name_row.set_content(&vulkan_info.device_name);
        self.version_row.set_content(&vulkan_info.api_version);

        self.driver_name_row
            .set_content(&vulkan_info.driver.name.as_deref().unwrap_or_default());

        self.driver_version_row
            .set_content(&vulkan_info.driver.info.as_deref().unwrap_or_default());

        self.features_model.remove_all();
        for (name, supported) in &vulkan_info.features {
            let feature = VulkanFeature::new(name.to_string(), *supported);
            self.features_model.append(&feature);
        }

        self.extensions_model.remove_all();
        for (name, supported) in &vulkan_info.extensions {
            let extension = VulkanFeature::new(name.to_string(), *supported);
            self.extensions_model.append(&extension);
        }
    }
}

fn show_features_window(title: &str, model: gio::ListStore) {
    let window = VulkanFeaturesWindow::new(title, model.into());
    window.present();
}
