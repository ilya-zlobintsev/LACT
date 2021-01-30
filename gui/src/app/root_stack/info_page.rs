use daemon::gpu_controller::GpuInfo;
use gtk::*;
pub struct InformationPage {
    pub container: Box,
    gpu_name_label: Label,
    gpu_manufacturer_label: Label,
}

impl InformationPage {
    pub fn new() -> Self {
        let container = Box::new(Orientation::Vertical, 5);
        
        let gpu_name_label = Label::new(None);
        let gpu_manufacturer_label = Label::new(None);
        
        container.add(&gpu_name_label);
        container.add(&gpu_manufacturer_label);

        Self { container, gpu_name_label, gpu_manufacturer_label }
    }
    
    pub fn set_info(&self, gpu_info: GpuInfo) {
        self.gpu_name_label.set_text(&gpu_info.vendor_data.card_model.unwrap_or_default());
        self.gpu_manufacturer_label.set_text(&gpu_info.vendor_data.card_vendor.unwrap_or_default());
    }
}