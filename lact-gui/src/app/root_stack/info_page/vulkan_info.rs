use gtk::prelude::*;
use gtk::*;
use lact_schema::VulkanInfo;
use std::collections::BTreeMap;
use tracing::trace;

#[derive(Clone)]
pub struct VulkanInfoFrame {
    pub container: Frame,
    device_name_label: Label,
    version_label: Label,
    features_box: Box,
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

        let grid = Grid::new();

        grid.set_margin_start(5);
        grid.set_margin_end(5);
        grid.set_margin_bottom(5);
        grid.set_margin_top(5);

        grid.set_column_homogeneous(true);

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

        let features_expander = Expander::new(Some("Feature support"));

        grid.attach(&features_expander, 0, 2, 5, 1);

        let features_scrolled_window = ScrolledWindow::new(NONE_ADJUSTMENT, NONE_ADJUSTMENT);

        features_scrolled_window.set_vexpand(true);

        let features_box = Box::new(Orientation::Vertical, 5);

        features_box.set_halign(Align::Center);

        features_scrolled_window.add(&features_box);

        features_expander.add(&features_scrolled_window);

        container.add(&grid);

        Self {
            container,
            device_name_label,
            version_label,
            features_box,
        }
    }

    pub fn set_info(&self, vulkan_info: &VulkanInfo) {
        trace!("Setting vulkan info: {:?}", vulkan_info);

        self.device_name_label
            .set_markup(&format!("<b>{}</b>", vulkan_info.device_name));
        self.version_label
            .set_markup(&format!("<b>{}</b>", vulkan_info.api_version));

        let features: BTreeMap<_, _> = vulkan_info.supported_features.iter().collect();
        for (feature, supported) in features.into_iter() {
            let vbox = Box::new(Orientation::Horizontal, 5);

            let feature_name_label = Label::new(Some(feature));

            vbox.pack_start(&feature_name_label, false, false, 0);

            let feature_supported_checkbutton = CheckButton::new();

            feature_supported_checkbutton.set_sensitive(false);
            feature_supported_checkbutton.set_active(*supported);

            vbox.pack_start(&feature_supported_checkbutton, false, false, 0);

            self.features_box.pack_end(&vbox, false, false, 0);
        }
    }
}
