use super::PageUpdate;
use crate::app::info_row::InfoRowItem;
use crate::app::page_section::PageSection;
use crate::I18N;
use gtk::prelude::*;
use i18n_embed_fl::fl;
use lact_schema::{DeviceInfo, DeviceStats};
use relm4::{prelude::FactoryVecDeque, ComponentParts, ComponentSender, RelmWidgetExt};
use std::sync::Arc;

pub struct InformationPage {
    values_list: FactoryVecDeque<InfoRowItem>,
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
                    }
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
        self.values_list.guard().clear();

        if let Some(info) = &self.device_info {
            let mut values_list = self.values_list.guard();
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
        }
    }
}
