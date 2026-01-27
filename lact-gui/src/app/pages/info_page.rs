use super::PageUpdate;
use crate::app::ext::FlowBoxExt;
use crate::app::formatting::fmt_human_bytes;
use crate::app::info_row::{InfoRow, InfoRowExt, InfoRowItem};
use crate::app::page_section::PageSection;
use crate::I18N;
use gtk::prelude::*;
use i18n_embed_fl::fl;
use lact_schema::{CacheInfo, CacheType, DeviceInfo, DeviceStats};
use relm4::{
    prelude::{FactoryComponent, FactoryVecDeque},
    ComponentParts, ComponentSender, RelmWidgetExt,
};
use std::sync::Arc;

pub struct InformationPage {
    values_list: FactoryVecDeque<InfoRowItem>,
    cache_list: FactoryVecDeque<CacheRow>,
    device_info: Option<Arc<DeviceInfo>>,
    device_stats: Option<Arc<DeviceStats>>,
}

#[relm4::component(pub)]
impl relm4::SimpleComponent for InformationPage {
    type Init = ();
    type Input = PageUpdate;
    type Output = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 15,
            set_margin_vertical: 15,
            set_margin_horizontal: 30,

            PageSection::new(&fl!(I18N, "hardware-info")) {
                append_child = &model.values_list.widget().clone() -> gtk::FlowBox {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_column_spacing: 10,
                    // set_homogeneous: true,
                    set_min_children_per_line: 2,
                    set_max_children_per_line: 4,
                    set_selection_mode: gtk::SelectionMode::None,

                    append_child = &InfoRow {
                        set_value: fl!(I18N, "cache-info"),
                        set_icon: "go-down-symbolic".to_string(),

                        #[name = "cache_popover"]
                        set_popover = &gtk::Popover {
                            model.cache_list.widget().clone() -> gtk::ListBox {
                                set_margin_all: 10,
                                set_selection_mode: gtk::SelectionMode::None,
                            },
                        },

                        connect_clicked[cache_popover] => move |_| {
                            cache_popover.popup();
                        },
                    } -> cache_row: gtk::FlowBoxChild {
                        #[watch]
                        set_visible: !model.cache_list.is_empty(),
                    },
                },
            },
        },
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            values_list: FactoryVecDeque::builder().launch_default().detach(),
            cache_list: FactoryVecDeque::builder().launch_default().detach(),
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

impl InformationPage {
    fn update_items(&mut self) {
        let mut values_list = self.values_list.guard();
        values_list.clear();
        let mut cache_list = self.cache_list.guard();
        cache_list.clear();

        if let Some(info) = &self.device_info {
            for (name, value) in info.info_elements(self.device_stats.as_deref()) {
                if let Some(value) = value {
                    let note = if name == "Card Model" && !value.starts_with("Unknown ") {
                        Some("The card displayed here may be of a sibling model, e.g. XT vs XTX variety. This is normal, as such models often use the same device ID, and it is not possible to differentiate between them.")
                    } else {
                        None
                    };

                    values_list.push_back(InfoRowItem { name, value, note });
                }
            }

            if let Some(drm_info) = &info.drm_info {
                if let Some(cache_info) = &drm_info.cache_info {
                    match cache_info {
                        CacheInfo::Amd(items) => {
                            for (instance, count) in items {
                                let cache_types = instance
                                    .types
                                    .iter()
                                    .map(|cache_type| match cache_type {
                                        CacheType::Data => fl!(I18N, "cache-data"),
                                        CacheType::Instruction => fl!(I18N, "cache-instruction"),
                                        CacheType::Cpu => fl!(I18N, "cache-cpu"),
                                    })
                                    .collect::<Vec<String>>()
                                    .join("+");

                                cache_list.push_back(CacheRow {
                                    count: *count,
                                    text: fl!(
                                        I18N,
                                        "amd-cache-desc",
                                        size = fmt_human_bytes(instance.size.into(), None),
                                        level = instance.level,
                                        types = cache_types,
                                        shared = instance.cu_count
                                    ),
                                });
                            }
                        }
                        CacheInfo::Nvidia { l2 } => {
                            cache_list.push_back(CacheRow {
                                count: 1,
                                text: fl!(
                                    I18N,
                                    "nvidia-cache-desc",
                                    size = fmt_human_bytes((*l2).into(), None),
                                    level = 2,
                                ),
                            });
                        }
                    }
                }
            }
        }
    }
}

struct CacheRow {
    count: u16,
    text: String,
}

#[relm4::factory]
impl FactoryComponent for CacheRow {
    type ParentWidget = gtk::ListBox;
    type Init = Self;
    type Input = ();
    type Output = ();
    type CommandOutput = ();

    fn init_model(
        init: Self::Init,
        _index: &Self::Index,
        _sender: relm4::FactorySender<Self>,
    ) -> Self {
        init
    }

    view! {
        gtk::ListBoxRow {
            set_activatable: false,
            set_selectable: false,

            gtk::Label {
                set_label: &format!("{}x {}", self.count, self.text),
                set_selectable: true,
                set_halign: gtk::Align::Start,
                set_margin_all: 5,
            }
        }
    }
}
