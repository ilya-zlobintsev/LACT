use super::{apply_box::ApplyBox, gpu_selector::GpuSelector};

#[derive(Debug, Clone)]
pub struct Headerbar {
    pub container: libadwaita::HeaderBar,
    pub gpu_selector: GpuSelector,
    pub apply_box: ApplyBox,
}

impl Headerbar {
    pub fn new() -> Self {
        let container = libadwaita::HeaderBar::builder().show_title(false).build();
        let gpu_selector = GpuSelector::new();
        let apply_box = ApplyBox::new();

        container.pack_start(&gpu_selector.dropdown);
        container.pack_end(&apply_box.container);

        Self {
            container,
            gpu_selector,
            apply_box,
        }
    }
}
