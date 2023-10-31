mod feature_model;

use crate::app::root_stack::{label_row, values_row};

use self::feature_model::FeatureModel;
use super::values_grid;
use glib::clone;
use gtk::prelude::*;
use gtk::*;
use lact_client::schema::VulkanInfo;
use std::cell::RefCell;
use std::rc::Rc;
use tracing::trace;

#[derive(Clone)]
pub struct VulkanInfoFrame {
    pub container: Box,
    device_name_label: Label,
    version_label: Label,
    driver_name_label: Label,
    driver_version_label: Label,
    features: Rc<RefCell<Vec<FeatureModel>>>,
    extensions: Rc<RefCell<Vec<FeatureModel>>>,
}

impl VulkanInfoFrame {
    pub fn new() -> Self {
        let container = Box::new(Orientation::Vertical, 0);

        let features = Rc::new(RefCell::new(Vec::new()));
        let extensions = Rc::new(RefCell::new(Vec::new()));

        let grid = values_grid();

        let device_name_label = label_row("Device name", &grid, 0, 0, true);
        let version_label = label_row("Vulkan version:", &grid, 1, 0, true);
        let driver_name_label = label_row("Driver name:", &grid, 2, 0, true);
        let driver_version_label = label_row("Driver version:", &grid, 3, 0, true);

        let show_features_button = Button::builder().label("Show").halign(Align::End).build();
        show_features_button.connect_clicked(clone!(@strong features => move |_| {
            show_list_window("Vulkan features", &features.borrow());
        }));
        values_row("Features:", &grid, &show_features_button, 4, 0);

        let show_extensions_button = Button::builder().label("Show").halign(Align::End).build();
        show_extensions_button.connect_clicked(clone!(@strong extensions => move |_| {
            show_list_window("Vulkan extensions", &extensions.borrow());
        }));
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

        let features_vec: Vec<_> = vulkan_info
            .features
            .iter()
            .map(|(name, supported)| FeatureModel::new(name.to_string(), *supported))
            .collect();
        self.features.replace(features_vec);

        let extensions_vec: Vec<_> = vulkan_info
            .extensions
            .iter()
            .map(|(name, supported)| FeatureModel::new(name.to_string(), *supported))
            .collect();
        self.extensions.replace(extensions_vec);
    }
}

fn show_list_window(title: &str, items: &[FeatureModel]) {
    let window = Window::builder()
        .title(title)
        .width_request(500)
        .height_request(700)
        .resizable(false)
        .build();

    let base_model = gio::ListStore::new::<FeatureModel>();
    base_model.extend_from_slice(items);

    let expression = PropertyExpression::new(FeatureModel::static_type(), Expression::NONE, "name");
    let filter = StringFilter::builder()
        .match_mode(StringFilterMatchMode::Substring)
        .ignore_case(true)
        .expression(expression)
        .build();

    let entry = SearchEntry::builder().hexpand(true).build();
    entry.connect_search_changed(clone!(@weak filter => move |entry| {
        if entry.text().is_empty() {
            filter.set_search(None);
        } else {
            filter.set_search(Some(entry.text().as_str()));
        }
    }));

    let filter_model = FilterListModel::builder()
        .model(&base_model)
        .filter(&filter)
        .incremental(true)
        .build();

    let selection_model = NoSelection::new(Some(filter_model));

    let factory = gtk::SignalListItemFactory::new();

    factory.connect_setup(move |_factory, item| {
        let item = item.downcast_ref::<gtk::ListItem>().unwrap();
        let label = Label::builder()
            .margin_top(5)
            .margin_bottom(5)
            .selectable(true)
            .hexpand(true)
            .halign(Align::Start)
            .build();
        let image = Image::new();
        let vbox = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(5)
            .margin_start(10)
            .margin_end(10)
            .build();
        vbox.append(&label);
        vbox.append(&image);
        item.set_child(Some(&vbox));
    });

    factory.connect_bind(move |_factory, item| {
        let item = item.downcast_ref::<gtk::ListItem>().unwrap();
        let model = item.item().and_downcast::<FeatureModel>().unwrap();

        let vbox = item.child().and_downcast::<Box>().unwrap();
        let children = vbox.observe_children();
        let label = children.item(0).and_downcast::<Label>().unwrap();
        let image = children.item(1).and_downcast::<Image>().unwrap();

        let text = model.property::<String>("name");
        let supported = model.property::<bool>("supported");
        label.set_text(&text);

        let icon_name = if supported {
            "emblem-ok-symbolic"
        } else {
            "action-unavailable-symbolic"
        };
        image.set_icon_name(Some(icon_name));
    });

    let list_view = ListView::new(Some(selection_model), Some(factory));
    let scroll_window = ScrolledWindow::builder()
        .child(&list_view)
        .vexpand(true)
        .build();

    let vbox = Box::new(Orientation::Vertical, 5);
    vbox.append(&entry);
    vbox.append(&scroll_window);

    window.set_child(Some(&vbox));
    window.present();
}
