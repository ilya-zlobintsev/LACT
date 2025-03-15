use crate::app::{info_row::InfoRow, page_section::PageSection, pages::PageUpdate};
use gtk::prelude::{BoxExt, OrientableExt, WidgetExt};
use lact_schema::{DeviceInfo, DeviceStats, DrmInfo};
use relm4::RelmWidgetExt;
use relm4::{prelude::FactoryVecDeque, ComponentParts, ComponentSender};
use std::fmt::Write;
use std::sync::Arc;

pub struct HardwareInfoSection {
    values_list: FactoryVecDeque<InfoRowItem>,
    device_info: Option<Arc<DeviceInfo>>,
    device_stats: Option<Arc<DeviceStats>>,
}

#[relm4::component(pub)]
impl relm4::SimpleComponent for HardwareInfoSection {
    type Init = ();
    type Input = PageUpdate;
    type Output = ();

    view! {
        PageSection::new("Hardware Information") {
            set_margin_start: 15,

            append = &model.values_list.widget().clone() -> gtk::Box {
                set_spacing: 10,
                set_orientation: gtk::Orientation::Vertical,
                set_margin_horizontal: 5,
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            values_list: FactoryVecDeque::builder().launch_default().detach(),
            device_info: None,
            device_stats: None,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            PageUpdate::Info(device_info) => {
                self.device_info = Some(device_info);
            }
            PageUpdate::Stats(device_stats) => {
                self.device_stats = Some(device_stats);
            }
        }
        self.update_items();
    }
}

impl HardwareInfoSection {
    fn update_items(&mut self) {
        self.values_list.guard().clear();

        if let Some(info) = &self.device_info {
            let pci_info = info.pci_info.as_ref();

            let mut gpu_model = info
                .drm_info
                .as_ref()
                .and_then(|drm| drm.device_name.as_deref())
                .or_else(|| pci_info.and_then(|pci_info| pci_info.device_pci_info.model.as_deref()))
                .unwrap_or("Unknown")
                .to_owned();

            let mut card_manufacturer = pci_info
                .and_then(|info| info.subsystem_pci_info.vendor.as_deref())
                .unwrap_or("Unknown")
                .to_owned();

            let mut card_model = pci_info
                .and_then(|info| info.subsystem_pci_info.model.as_deref())
                .unwrap_or("Unknown")
                .to_owned();

            if let Some(pci_info) = &info.pci_info {
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

                let _ = write!(
                    card_manufacturer,
                    " (0x{})",
                    pci_info.subsystem_pci_info.vendor_id
                );

                let _ = write!(card_model, " (0x{})", pci_info.subsystem_pci_info.model_id);
            };

            let mut elements = vec![
                ("GPU Model", Some(gpu_model)),
                ("Card Manufacturer", Some(card_manufacturer)),
                ("Card Model", Some(card_model)),
                ("Driver Used", Some(info.driver.clone())),
                ("VBIOS Version", info.vbios_version.clone()),
            ];

            if let Some(stats) = &self.device_stats {
                elements.push((
                    "VRAM Size",
                    stats
                        .vram
                        .total
                        .map(|size| format!("{} MiB", size / 1024 / 1024)),
                ));
            }

            if let Some(drm_info) = &info.drm_info {
                elements.extend([
                    ("GPU Family", drm_info.family_name.clone()),
                    ("ASIC Name", drm_info.asic_name.clone()),
                    (
                        "Compute Units",
                        drm_info.compute_units.map(|count| count.to_string()),
                    ),
                    (
                        "Execution Units",
                        drm_info
                            .intel
                            .execution_units
                            .map(|count| count.to_string()),
                    ),
                    (
                        "Subslices",
                        drm_info
                            .intel
                            .execution_units
                            .map(|count| count.to_string()),
                    ),
                    (
                        "Cuda Cores",
                        drm_info.cuda_cores.map(|count| count.to_string()),
                    ),
                    (
                        "Streaming Multiprocessors",
                        drm_info
                            .streaming_multiprocessors
                            .map(|count| count.to_string()),
                    ),
                    (
                        "ROP Count",
                        drm_info.rop_info.as_ref().map(|rop| {
                            format!(
                                "{} ({} * {})",
                                rop.operations_count, rop.unit_count, rop.operations_factor
                            )
                        }),
                    ),
                    ("VRAM Type", drm_info.vram_type.clone()),
                    ("VRAM Manufacturer", drm_info.vram_vendor.clone()),
                    ("Theoretical VRAM Bandwidth", drm_info.vram_max_bw.clone()),
                    (
                        "L1 Cache (Per CU)",
                        drm_info
                            .l1_cache_per_cu
                            .map(|cache| format!("{} KiB", cache / 1024)),
                    ),
                    (
                        "L2 Cache",
                        drm_info
                            .l2_cache
                            .map(|cache| format!("{} KiB", cache / 1024)),
                    ),
                    (
                        "L3 Cache",
                        drm_info.l3_cache_mb.map(|cache| format!("{cache} MiB")),
                    ),
                ]);

                if let Some(memory_info) = &drm_info.memory_info {
                    if let Some(rebar) = memory_info.resizeable_bar {
                        let rebar = if rebar { "Enabled" } else { "Disabled" };
                        elements.push(("Resizeable bar", Some(rebar.to_owned())));
                    }

                    elements.push((
                        "CPU Accessible VRAM",
                        Some((memory_info.cpu_accessible_total / 1024 / 1024).to_string()),
                    ));
                }
            }

            if let (Some(link_speed), Some(link_width)) =
                (&info.link_info.current_speed, &info.link_info.current_width)
            {
                elements.push(("Link Speed", Some(format!("{link_speed} x{link_width}"))));
            }

            let mut values_list = self.values_list.guard();
            for (name, value) in elements {
                if let Some(value) = value {
                    let note = if name == "Card Model" && !value.starts_with("Unknown ") {
                        Some("The card displayed here may be of a sibling model, e.g. XT vs XTX variety. This is normal, as such models often use the same device ID, and it is not possible to differentiate between them.)")
                    } else {
                        None
                    };

                    values_list.push_back(InfoRowItem {
                        name: format!("{name}:"),
                        value,
                        note,
                    });
                }
            }
        }
    }
}

struct InfoRowItem {
    name: String,
    value: String,
    note: Option<&'static str>,
}

#[relm4::factory]
impl relm4::factory::FactoryComponent for InfoRowItem {
    type Init = Self;
    type ParentWidget = gtk::Box;
    type CommandOutput = ();
    type Input = ();
    type Output = ();

    fn init_model(
        init: Self::Init,
        _index: &Self::Index,
        _sender: relm4::FactorySender<Self>,
    ) -> Self {
        init
    }

    view! {
        InfoRow {
            set_selectable: true,
            set_name: self.name.clone(),
            set_value: self.value.clone(),
            set_info_text: self.note.unwrap_or_default(),
        }
    }
}
