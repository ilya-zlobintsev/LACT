mod feature_window;

use super::values_grid;
use crate::app::pages::{label_row, values_row};
use feature_window::{VulkanFeature, VulkanFeaturesWindow};
use glib::clone;
use gtk::prelude::*;
use gtk::*;
use lact_client::schema::VulkanInfo;
use relm4::{Component, ComponentController};
use std::cell::RefCell;
use std::rc::Rc;
use tracing::trace;

#[derive(Clone, Debug)]
pub struct VulkanInfoFrame {
    pub container: Box,
    device_name_label: Label,
    version_label: Label,
    driver_name_label: Label,
    driver_version_label: Label,
    features: Rc<RefCell<Vec<VulkanFeature>>>,
    extensions: Rc<RefCell<Vec<VulkanFeature>>>,
}

impl VulkanInfoFrame {
    pub fn new() -> Self {
        let container = Box::new(Orientation::Vertical, 0);

        let features: Rc<RefCell<Vec<VulkanFeature>>> = Rc::default();
        let extensions: Rc<RefCell<Vec<VulkanFeature>>> = Rc::default();

        let grid = values_grid();
        grid.set_margin_start(0);
        grid.set_margin_end(0);

        let device_name_label = label_row("Device name", &grid, 0, 0, true);
        let version_label = label_row("Vulkan version:", &grid, 1, 0, true);
        let driver_name_label = label_row("Driver name:", &grid, 2, 0, true);
        let driver_version_label = label_row("Driver version:", &grid, 3, 0, true);

        let show_features_button = Button::builder().label("Show").halign(Align::End).build();
        show_features_button.connect_clicked(clone!(
            #[strong]
            features,
            move |_| {
                show_features_window("Vulkan features", features.clone());
            }
        ));
        values_row("Features:", &grid, &show_features_button, 4, 0);

        let show_extensions_button = Button::builder().label("Show").halign(Align::End).build();
        show_extensions_button.connect_clicked(clone!(
            #[strong]
            extensions,
            move |_| {
                show_features_window("Vulkan extensions", extensions.clone());
            }
        ));
        values_row("Extensions:", &grid, &show_extensions_button, 5, 0);

        container.append(&grid);

        Self {
            container,
            device_name_label,
            version_label,
            driver_name_label,
            driver_version_label,
            features,
            extensions,
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

        let mut features = self.features.borrow_mut();
        features.clear();

        for (name, supported) in &vulkan_info.features {
            let feature = VulkanFeature {
                name: name.to_string(),
                supported: *supported,
            };
            features.push(feature);
        }

        let mut extensions = self.extensions.borrow_mut();
        extensions.clear();
        for (name, supported) in &vulkan_info.extensions {
            let extension = VulkanFeature {
                name: name.to_string(),
                supported: *supported,
            };
            extensions.push(extension);
        }
    }
}

fn show_features_window(title: &str, values: Rc<RefCell<Vec<VulkanFeature>>>) {
    let features = values.borrow().iter().cloned().collect();

    let mut window_controller = VulkanFeaturesWindow::builder()
        .launch((features, title.to_owned()))
        .detach();
    window_controller.detach_runtime();
    window_controller.widget().present();
}
