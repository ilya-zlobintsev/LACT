use glib::clone;
use gtk::prelude::*;
use gtk::*;
use lact_client::schema::VulkanInfo;
use tracing::trace;

#[derive(Clone)]
pub struct VulkanInfoFrame {
    pub container: Frame,
    device_name_label: Label,
    version_label: Label,
    features_listbox: ListBox,
    extensions_listbox: ListBox,
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

        // container.set_shadow_type(ShadowType::None); // TODO

        let features_listbox = ListBox::builder().halign(Align::Fill).build();
        let extensions_listbox = ListBox::builder().halign(Align::Fill).build();

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
        show_features_button.connect_clicked(clone!(@strong features_listbox => move |_| {
            show_list_window("Vulkan features", &features_listbox);
        }));

        grid.attach(&features_label, 0, 2, 2, 1);
        grid.attach(&show_features_button, 2, 2, 2, 1);

        let extensions_label = Label::builder()
            .label("Extensions:")
            .halign(Align::End)
            .build();
        let show_extensions_button = Button::builder().label("Show").halign(Align::Start).build();
        show_extensions_button.connect_clicked(clone!(@strong extensions_listbox => move |_| {
            show_list_window("Vulkan extensions", &extensions_listbox);
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
            features_listbox,
            extensions_listbox,
        }
    }

    pub fn set_info(&self, vulkan_info: &VulkanInfo) {
        trace!("Setting vulkan info: {:?}", vulkan_info);

        self.device_name_label
            .set_markup(&format!("<b>{}</b>", vulkan_info.device_name));
        self.version_label
            .set_markup(&format!("<b>{}</b>", vulkan_info.api_version));

        // self.features_listbox.children().clear();
        for (i, (feature, supported)) in vulkan_info.features.iter().enumerate() {
            let vbox = Box::new(Orientation::Horizontal, 5);

            let feature_name_label = Label::new(Some(&feature));

            vbox.append(&feature_name_label);

            let feature_supported_checkbutton = CheckButton::new();

            feature_supported_checkbutton.set_sensitive(false);
            feature_supported_checkbutton.set_active(*supported);

            vbox.append(&feature_supported_checkbutton);

            self.features_listbox.insert(&vbox, i.try_into().unwrap());
        }

        // self.extensions_listbox.children().clear();
        for (i, (extension, supported)) in vulkan_info.extensions.iter().enumerate() {
            let vbox = Box::new(Orientation::Horizontal, 5);
            vbox.set_hexpand(true);

            let extension_name_label = Label::new(Some(&extension));
            vbox.append(&extension_name_label);

            let extension_supported_checkbutton = CheckButton::builder()
                .sensitive(false)
                .active(*supported)
                .build();
            vbox.append(&extension_supported_checkbutton);

            self.extensions_listbox.insert(&vbox, i.try_into().unwrap());
        }
    }
}

fn show_list_window(title: &str, child: &ListBox) {
    let window = Window::builder()
        .title(title)
        .width_request(500)
        .height_request(700)
        .build();
    let scroll = ScrolledWindow::builder().child(child).build();
    window.set_child(Some(&scroll));
    window.show();
}
