use gtk::glib::{self, object::ObjectExt, subclass::object::DerivedObjectProperties, Object};
use lact_client::schema::{DeviceInfo, DeviceStats};

glib::wrapper! {
    pub struct HardwareInfoSection(ObjectSubclass<imp::HardwareInfoSection>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Orientable, gtk::Accessible, gtk::Buildable;
}

impl HardwareInfoSection {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_info(&self, info: &DeviceInfo) {
        self.reset();

        if let Some(pci_info) = &info.pci_info {
            if let Some(name) = pci_info
                .subsystem_pci_info
                .model
                .as_deref()
                .or(pci_info.device_pci_info.model.as_deref())
            {
                self.set_gpu_model(name);
            }

            if let Some(manufacturer_name) = info.pci_info.as_ref().and_then(|pci_info| {
                pci_info
                    .subsystem_pci_info
                    .vendor
                    .as_deref()
                    .or(pci_info.device_pci_info.model.as_deref())
            }) {
                self.set_gpu_manufacturer(manufacturer_name);
            }
        }

        if let Some(drm_info) = &info.drm_info {
            self.set_gpu_family(drm_info.family_name.clone());
            self.set_asic_name(drm_info.asic_name.clone());
            self.set_compute_units(drm_info.compute_units.to_string());

            self.set_vram_type(drm_info.vram_type.clone());
            self.set_peak_vram_bandwidth(format!("{} GiB/s", drm_info.vram_max_bw));
            self.set_l1_cache(format!("{} KiB", drm_info.l1_cache_per_cu / 1024));
            self.set_l2_cache(format!("{} KiB", drm_info.l2_cache / 1024));
            self.set_l3_cache(format!("{} MiB", drm_info.l3_cache_mb));

            if let Some(memory_info) = &drm_info.memory_info {
                let rebar = if memory_info.resizeable_bar {
                    "Enabled"
                } else {
                    "Disabled"
                };
                self.set_rebar(rebar);

                self.set_cpu_accessible_vram(format!(
                    "{} MiB",
                    memory_info.cpu_accessible_total / 1024 / 1024
                ));
            }
        }

        self.set_driver_used(info.driver);

        if let Some(vbios) = &info.vbios_version {
            self.set_vbios_version(vbios.clone());
        }

        if let (Some(link_speed), Some(link_width)) =
            (&info.link_info.current_speed, &info.link_info.current_width)
        {
            self.set_link_speed(format!("{link_speed} x{link_width}",));
        }
    }

    pub fn set_stats(&self, stats: &DeviceStats) {
        if let Some(total_vram) = stats.vram.total {
            self.set_vram_size(format!("{} MiB", total_vram / 1024 / 1024));
        }
    }

    fn reset(&self) {
        let properties = imp::HardwareInfoSection::derived_properties();
        for property in properties {
            self.set_property(property.name(), "Unknown");
        }
    }
}

impl Default for HardwareInfoSection {
    fn default() -> Self {
        Object::builder().build()
    }
}

mod imp {
    #![allow(clippy::enum_variant_names)]
    use crate::app::{info_row::InfoRow, page_section::PageSection};
    use glib::Properties;
    use gtk::{
        glib::{self, subclass::InitializingObject},
        prelude::*,
        subclass::{
            prelude::*,
            widget::{CompositeTemplateClass, WidgetImpl},
        },
        CompositeTemplate,
    };
    use std::cell::RefCell;

    #[derive(CompositeTemplate, Default, Properties)]
    #[properties(wrapper_type = super::HardwareInfoSection)]
    #[template(file = "ui/info_page/hardware_info_section.blp")]
    pub struct HardwareInfoSection {
        #[property(get, set)]
        gpu_model: RefCell<String>,
        #[property(get, set)]
        gpu_manufacturer: RefCell<String>,
        #[property(get, set)]
        gpu_family: RefCell<String>,
        #[property(get, set)]
        asic_name: RefCell<String>,
        #[property(get, set)]
        compute_units: RefCell<String>,
        #[property(get, set)]
        vbios_version: RefCell<String>,
        #[property(get, set)]
        driver_used: RefCell<String>,
        #[property(get, set)]
        vram_size: RefCell<String>,
        #[property(get, set)]
        vram_type: RefCell<String>,
        #[property(get, set)]
        peak_vram_bandwidth: RefCell<String>,
        #[property(get, set)]
        l1_cache: RefCell<String>,
        #[property(get, set)]
        l2_cache: RefCell<String>,
        #[property(get, set)]
        l3_cache: RefCell<String>,
        #[property(get, set)]
        rebar: RefCell<String>,
        #[property(get, set)]
        cpu_accessible_vram: RefCell<String>,
        #[property(get, set)]
        link_speed: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for HardwareInfoSection {
        const NAME: &'static str = "HardwareInfoSection";
        type Type = super::HardwareInfoSection;
        type ParentType = PageSection;

        fn class_init(class: &mut Self::Class) {
            InfoRow::ensure_type();
            PageSection::ensure_type();

            class.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for HardwareInfoSection {}

    impl WidgetImpl for HardwareInfoSection {}
    impl BoxImpl for HardwareInfoSection {}
}
