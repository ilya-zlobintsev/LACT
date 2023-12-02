use super::{apply_box::ApplyBox, gpu_selector::GpuSelector};

#[derive(Debug, Clone)]
pub struct Headerbar {
    #[cfg(feature = "adw")]
    pub container: adw::HeaderBar,

    #[cfg(not(feature = "adw"))]
    pub container: gtk::HeaderBar,

    pub gpu_selector: GpuSelector,
    pub apply_box: ApplyBox,
}

impl Headerbar {
    pub fn new() -> Self {
        #[cfg(feature = "adw")]
        let container = adw::HeaderBar::builder().show_title(false).build();

        #[cfg(not(feature = "adw"))]
        let container = gtk::HeaderBar::builder()
            .title_widget(&gtk::Label::new(None))
            .build();

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
