use crate::app::page_section::PageSection;
use gtk::glib::{self, object::ObjectExt, subclass::object::DerivedObjectProperties, Object};
use lact_client::schema::{DeviceInfo, DeviceStats, DrmInfo};
use std::fmt::Write;

glib::wrapper! {
    pub struct HardwareInfoSection(ObjectSubclass<imp::HardwareInfoSection>)
        @extends gtk::Box, gtk::Widget, PageSection,
        @implements gtk::Orientable, gtk::Accessible, gtk::Buildable;
}

impl HardwareInfoSection {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_info(&self, info: &DeviceInfo) {
        self.reset();

        if let Some(pci_info) = &info.pci_info {
            let mut gpu_model = info
                .drm_info
                .as_ref()
                .and_then(|drm| drm.device_name.as_deref())
                .or_else(|| pci_info.device_pci_info.model.as_deref())
                .unwrap_or("Unknown")
                .to_owned();

            match &info.drm_info {
                Some(DrmInfo {
                    pci_revision_id: Some(pci_rev),
                    ..
                }) => {
                    let _ = write!(
                        gpu_model,
                        " (0x{}:0x{}:0x{pci_rev:X})",
                        pci_info.device_pci_info.vendor_id, pci_info.device_pci_info.model_id,
                    );
                }
                _ => {
                    let _ = write!(
                        gpu_model,
                        " (0x{}:0x{})",
                        pci_info.device_pci_info.vendor_id, pci_info.device_pci_info.model_id
                    );
                }
            }

            self.set_gpu_model(gpu_model);

            let mut card_manufacturer = pci_info
                .subsystem_pci_info
                .vendor
                .as_deref()
                .unwrap_or("Unknown")
                .to_owned();
            let _ = write!(
                card_manufacturer,
                " (0x{})",
                pci_info.subsystem_pci_info.vendor_id
            );
            self.set_card_manufacturer(card_manufacturer);

            let mut card_model = pci_info
                .subsystem_pci_info
                .model
                .as_deref()
                .unwrap_or("Unknown")
                .to_owned();
            let _ = write!(card_model, " (0x{})", pci_info.subsystem_pci_info.model_id);
            self.set_card_model(card_model);
        }

        if let Some(drm_info) = &info.drm_info {
            if let Some(family) = drm_info.family_name.as_deref() {
                self.set_gpu_family(family);
            }
            if let Some(asic) = drm_info.asic_name.as_deref() {
                self.set_asic_name(asic);
            }
            if let Some(units) = drm_info.compute_units {
                self.set_compute_units(units.to_string());
            }
            if let Some(cores) = drm_info.cuda_cores {
                self.set_cuda_cores(cores.to_string());
            }
            if let Some(vram_type) = drm_info.vram_type.as_deref() {
                self.set_vram_type(vram_type);
            }
            if let Some(max_bw) = &drm_info.vram_max_bw {
                self.set_peak_vram_bandwidth(format!("{max_bw} GiB/s"));
            }

            if let Some(l1) = drm_info.l1_cache_per_cu {
                self.set_l1_cache(format!("{} KiB", l1 / 1024));
            }
            if let Some(l2) = drm_info.l2_cache {
                self.set_l2_cache(format!("{} KiB", l2 / 1024));
            }
            if let Some(l3) = drm_info.l3_cache_mb {
                self.set_l3_cache(format!("{l3} MiB"));
            }

            if let Some(memory_info) = &drm_info.memory_info {
                if let Some(rebar) = memory_info.resizeable_bar {
                    let rebar = if rebar { "Enabled" } else { "Disabled" };
                    self.set_rebar(rebar);
                }

                self.set_cpu_accessible_vram(format!(
                    "{} MiB",
                    memory_info.cpu_accessible_total / 1024 / 1024
                ));
            }
        }

        self.set_driver_used(info.driver.as_str());

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
            self.set_property(property.name(), "");
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
        card_manufacturer: RefCell<String>,
        #[property(get, set)]
        card_model: RefCell<String>,
        #[property(get, set)]
        gpu_family: RefCell<String>,
        #[property(get, set)]
        asic_name: RefCell<String>,
        #[property(get, set)]
        compute_units: RefCell<String>,
        #[property(get, set)]
        cuda_cores: RefCell<String>,
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
    impl ObjectImpl for HardwareInfoSection {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            for child in obj.observe_children().into_iter().flatten() {
                if let Ok(row) = child.downcast::<InfoRow>() {
                    row.connect_value_notify(|row| {
                        row.set_visible(!row.value().is_empty());
                    });
                }
            }
        }
    }

    impl WidgetImpl for HardwareInfoSection {}
    impl BoxImpl for HardwareInfoSection {}
}
