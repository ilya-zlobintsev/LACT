mod feature_model;

use self::feature_model::FeatureModel;
use glib::clone;
use gtk::prelude::*;
use gtk::*;
use lact_client::schema::VulkanInfo;
use std::cell::RefCell;
use std::rc::Rc;
use tracing::trace;

#[derive(Clone)]
pub struct VulkanInfoFrame {
    pub container: Frame,
    device_name_label: Label,
    version_label: Label,
    features: Rc<RefCell<Vec<FeatureModel>>>,
    extensions: Rc<RefCell<Vec<FeatureModel>>>,
}

impl VulkanInfoFrame {
    pub fn new() -> Self {
        let container = Frame::new(None);

        container.set_label_widget(Some(&{
            let label = Label::new(None);
            label.set_markup("<span font_desc='11'><b>Vulkan Information</b></span>");
            label
        }));
        container.set_label_align(0.5);

        let features = Rc::new(RefCell::new(Vec::new()));
        let extensions = Rc::new(RefCell::new(Vec::new()));

        let vbox = Box::new(Orientation::Vertical, 5);

        let grid = Grid::new();

        grid.set_margin_start(5);
        grid.set_margin_end(5);
        grid.set_margin_bottom(5);
        grid.set_margin_top(5);

        grid.set_column_homogeneous(true);
        grid.set_row_homogeneous(false);

        grid.set_row_spacing(7);
        grid.set_column_spacing(5);

        grid.attach(
            &{
                let label = Label::new(Some("Device name:"));
                label.set_halign(Align::End);
                label
            },
            0,
            0,
            2,
            1,
        );

        let device_name_label = Label::new(None);
        device_name_label.set_halign(Align::Start);

        grid.attach(&device_name_label, 2, 0, 3, 1);

        grid.attach(
            &{
                let label = Label::new(Some("Version:"));
                label.set_halign(Align::End);
                label
            },
            0,
            1,
            2,
            1,
        );

        let version_label = Label::new(None);
        version_label.set_halign(Align::Start);

        grid.attach(&version_label, 2, 1, 3, 1);

        let features_label = Label::builder()
            .label("Features:")
            .halign(Align::End)
            .build();
        let show_features_button = Button::builder().label("Show").halign(Align::Start).build();
        show_features_button.connect_clicked(clone!(@strong features => move |_| {
            show_list_window("Vulkan features", &features.borrow());
        }));

        grid.attach(&features_label, 0, 2, 2, 1);
        grid.attach(&show_features_button, 2, 2, 2, 1);

        let extensions_label = Label::builder()
            .label("Extensions:")
            .halign(Align::End)
            .build();
        let show_extensions_button = Button::builder().label("Show").halign(Align::Start).build();
        show_extensions_button.connect_clicked(clone!(@strong extensions => move |_| {
            show_list_window("Vulkan extensions", &extensions.borrow());
        }));

        grid.attach(&extensions_label, 0, 3, 2, 1);
        grid.attach(&show_extensions_button, 2, 3, 2, 1);

        vbox.prepend(&grid);

        /*let features_expander = Expander::builder().label("Feature support").build();

        let features_scrolled_window = ScrolledWindow::builder().build();

        features_scrolled_window.set_vexpand(true);


        features_scrolled_window.add(&features_listbox);

        features_expander.add(&features_scrolled_window);

        vbox.pack_start(&features_expander, false, true, 5);


        let extensions_expander = Expander::builder()
            .label("Extension support")
            .child(
                &ScrolledWindow::builder()
                    .vexpand(true)
                    .child(&extensions_listbox)
                    .build(),
            )
            .build();

        vbox.pack_start(&extensions_expander, false, true, 5);*/

        container.set_child(Some(&vbox));

        Self {
            container,
            device_name_label,
            version_label,
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
        .build();

    let base_model = gio::ListStore::new(FeatureModel::static_type());
    base_model.extend_from_slice(&items);

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
    let search_bar = SearchBar::builder()
        .child(&entry)
        .search_mode_enabled(true)
        .key_capture_widget(&window)
        .build();

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
    vbox.append(&search_bar);
    vbox.append(&scroll_window);

    window.set_child(Some(&vbox));
    window.present();
}
