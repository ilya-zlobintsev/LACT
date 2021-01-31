use daemon::gpu_controller::VulkanInfo;
use gtk::*;

#[derive(Clone)]
pub struct VulkanInfoFrame {
    pub container: Frame,
    device_name_label: Label,
    version_label: Label,
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

        container.add(&grid);

        Self {
            container,
            device_name_label,
            version_label,
        }
    }

    pub fn set_info(&self, vulkan_info: &VulkanInfo) {
        self.device_name_label
            .set_markup(&format!("<b>{}</b>", vulkan_info.device_name));
        self.version_label
            .set_markup(&format!("<b>{}</b>", vulkan_info.api_version));
    }
}
