use gtk::prelude::*;
use gtk::*;
use lact_schema::VulkanInfo;
use tracing::trace;

#[derive(Clone)]
pub struct VulkanInfoFrame {
    pub container: Frame,
    device_name_label: Label,
    version_label: Label,
    features_box: Box,
    extensions_box: Box,
}

impl VulkanInfoFrame {
    pub fn new() -> Self {
        let container = Frame::new(None);

        container.set_label_widget(Some(&{
            let label = Label::new(None);
            label.set_markup("<span font_desc='11'><b>Vulkan Information</b></span>");
            label
        }));
        container.set_label_align(0.5, 0.5);

        container.set_shadow_type(ShadowType::None);

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

        vbox.pack_start(&grid, false, true, 5);

        let features_expander = Expander::builder().label("Feature support").build();

        let features_scrolled_window = ScrolledWindow::builder().build();

        features_scrolled_window.set_vexpand(true);

        let features_box = Box::new(Orientation::Vertical, 5);
        features_box.set_halign(Align::Center);
        features_box.set_valign(Align::Fill);

        features_scrolled_window.add(&features_box);

        features_expander.add(&features_scrolled_window);

        vbox.pack_start(&features_expander, false, true, 5);

        let extensions_box = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(5)
            .halign(Align::Center)
            .build();

        let extensions_expander = Expander::builder()
            .label("Extension support")
            .child(
                &ScrolledWindow::builder()
                    .vexpand(true)
                    .child(&extensions_box)
                    .build(),
            )
            .build();

        vbox.pack_start(&extensions_expander, false, true, 5);

        container.add(&vbox);

        Self {
            container,
            device_name_label,
            version_label,
            features_box,
            extensions_box,
        }
    }

    pub fn set_info(&self, vulkan_info: &VulkanInfo) {
        trace!("Setting vulkan info: {:?}", vulkan_info);

        self.device_name_label
            .set_markup(&format!("<b>{}</b>", vulkan_info.device_name));
        self.version_label
            .set_markup(&format!("<b>{}</b>", vulkan_info.api_version));

        for (feature, supported) in &vulkan_info.supported_features {
            let vbox = Box::new(Orientation::Horizontal, 5);

            let feature_name_label = Label::new(Some(&feature));

            vbox.pack_start(&feature_name_label, false, false, 0);

            let feature_supported_checkbutton = CheckButton::new();

            feature_supported_checkbutton.set_sensitive(false);
            feature_supported_checkbutton.set_active(*supported);

            vbox.pack_end(&feature_supported_checkbutton, false, false, 0);

            self.features_box.pack_start(&vbox, false, false, 0);
        }

        for (extension, supported) in &vulkan_info.supported_extensions {
            let vbox = Box::new(Orientation::Horizontal, 5);
            let extension_name_label = Label::new(Some(&extension));
            vbox.pack_start(&extension_name_label, false, false, 0);

            let extension_supported_checkbutton = CheckButton::builder()
                .sensitive(false)
                .active(*supported)
                .build();
            vbox.pack_end(&extension_supported_checkbutton, false, false, 0);

            self.extensions_box.pack_start(&vbox, false, false, 0);
        }
    }
}
