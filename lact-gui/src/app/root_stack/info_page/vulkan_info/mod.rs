mod feature_window;

use self::feature_window::VulkanFeaturesWindow;

use super::values_grid;
use crate::app::root_stack::info_page::vulkan_info::feature_window::feature::VulkanFeature;
use crate::app::root_stack::{label_row, values_row};
use glib::clone;
use gtk::prelude::*;
use gtk::*;
use lact_client::schema::VulkanInfo;
use tracing::trace;

#[derive(Clone)]
pub struct VulkanInfoFrame {
    pub container: Box,
    device_name_label: Label,
    version_label: Label,
    driver_name_label: Label,
    driver_version_label: Label,
    features_model: gio::ListStore,
    extensions_model: gio::ListStore,
}

impl VulkanInfoFrame {
    pub fn new() -> Self {
        let container = Box::new(Orientation::Vertical, 0);

        let features_model = gio::ListStore::new::<VulkanFeature>();
        let extensions_model = gio::ListStore::new::<VulkanFeature>();

        let grid = values_grid();

        let device_name_label = label_row("Device name", &grid, 0, 0, true);
        let version_label = label_row("Vulkan version:", &grid, 1, 0, true);
        let driver_name_label = label_row("Driver name:", &grid, 2, 0, true);
        let driver_version_label = label_row("Driver version:", &grid, 3, 0, true);

        let show_features_button = Button::builder().label("Show").halign(Align::End).build();
        show_features_button.connect_clicked(clone!(@strong features_model => move |_| {
            show_features_window("Vulkan features", features_model.clone());
        }));
        values_row("Features:", &grid, &show_features_button, 4, 0);

        let show_extensions_button = Button::builder().label("Show").halign(Align::End).build();
        show_extensions_button.connect_clicked(clone!(@strong extensions_model => move |_| {
            show_features_window("Vulkan extensions", extensions_model.clone());
        }));
        values_row("Extensions:", &grid, &show_extensions_button, 5, 0);

        container.append(&grid);

        Self {
            container,
            device_name_label,
            version_label,
            driver_name_label,
            driver_version_label,
            features_model,
            extensions_model,
        }
    }

    pub fn set_info(&self, vulkan_info: &VulkanInfo) {
        trace!("setting vulkan info: {:?}", vulkan_info);

        self.device_name_label
            .set_markup(&format!("<b>{}</b>", vulkan_info.device_name));
        self.version_label
            .set_markup(&format!("<b>{}</b>", vulkan_info.api_version));

        self.driver_name_label.set_markup(&format!(
            "<b>{}</b>",
            vulkan_info.driver.name.as_deref().unwrap_or_default(),
        ));

        self.driver_version_label.set_markup(&format!(
            "<b>{}</b>",
            vulkan_info.driver.info.as_deref().unwrap_or_default(),
        ));

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
