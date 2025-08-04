use super::PageUpdate;
use crate::app::format_friendly_size;
use crate::app::info_row::InfoRowItem;
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
        gtk::ScrolledWindow {
            set_hscrollbar_policy: gtk::PolicyType::Never,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 15,
                set_margin_horizontal: 20,

                PageSection::new(&fl!(I18N, "hardware-info")) {
                    append = &model.values_list.widget().clone() -> gtk::Box {
                        set_spacing: 10,
                        set_orientation: gtk::Orientation::Vertical,
                    },

                    append = &gtk::Expander {
                        set_label: Some(&fl!(I18N, "cache-info")),
                        #[watch]
                        set_visible: !model.cache_list.is_empty(),

                        gtk::Frame {
                            model.cache_list.widget().clone() -> gtk::Box {
                                set_spacing: 5,
                                set_margin_all: 10,
                                set_orientation: gtk::Orientation::Vertical,
                            },
                        }
                    },
                },
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
                                        size = format_friendly_size(instance.size.into()),
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
                                    size = format_friendly_size((*l2).into()),
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
    type ParentWidget = gtk::Box;
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
        gtk::Label {
            set_label: &format!("{}x {}", self.count, self.text),
            set_selectable: true,
            set_halign: gtk::Align::Start,
        }
    }
}
