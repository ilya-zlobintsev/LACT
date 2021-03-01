use gtk::*;

#[derive(Clone)]
pub struct WarningFrame {
    pub container: Frame,
}

impl WarningFrame {
    pub fn new() -> Self {
        let container = Frame::new(Some("Overclocking information"));

        container.set_label_align(0.3, 0.5);

        let warning_label = Label::new(None);

        warning_label.set_line_wrap(true);
        warning_label.set_markup("Overclocking support is not enabled! To enable overclocking support, you need to add <b>amdgpu.ppfeaturemask=0xffffffff</b> to your kernel boot options. Look for the documentation of your distro.");
        warning_label.set_selectable(true);

        container.add(&warning_label);

        Self { container }
    }

    pub fn show(&self) {
        self.container.set_visible(true);
    }

    pub fn hide(&self) {
        self.container.set_visible(false);
    }
}
