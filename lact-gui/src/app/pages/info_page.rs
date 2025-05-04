mod topology_window;

use super::PageUpdate;
use crate::app::{info_row::InfoRow, page_section::PageSection};
use gtk::prelude::*;
use lact_schema::{DeviceInfo, DeviceStats};
use relm4::{
    prelude::FactoryVecDeque, Component, ComponentController, ComponentParts, ComponentSender,
    RelmWidgetExt,
};
use std::sync::Arc;
use topology_window::{TopologyType, TopologyWindow};

pub struct InformationPage {
    values_list: FactoryVecDeque<InfoRowItem>,
    device_info: Option<Arc<DeviceInfo>>,
    device_stats: Option<Arc<DeviceStats>>,
    topology: Option<TopologyType>,
}

#[derive(Debug)]
pub enum InfoPageMsg {
    PageUpdate(PageUpdate),
    ShowTopology,
}

#[relm4::component(pub)]
impl relm4::SimpleComponent for InformationPage {
    type Init = ();
    type Input = InfoPageMsg;
    type Output = ();

    view! {
        gtk::ScrolledWindow {
            set_hscrollbar_policy: gtk::PolicyType::Never,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 15,
                set_margin_horizontal: 20,

                PageSection::new("Hardware Information") {
                    append = &model.values_list.widget().clone() -> gtk::Box {
                        set_spacing: 10,
                        set_orientation: gtk::Orientation::Vertical,
                    },

                    append = &gtk::Button {
                        set_label: "Show Topology",
                        #[watch]
                        set_visible: model.topology.is_some(),
                        connect_clicked => InfoPageMsg::ShowTopology,
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
            device_info: None,
            device_stats: None,
            topology: None,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            InfoPageMsg::PageUpdate(PageUpdate::Info(device_info)) => {
                let drm_info = device_info.drm_info.as_ref();

                self.topology = drm_info
                    .and_then(|info| info.intel.intel_topology.clone().map(TopologyType::Intel));

                self.device_info = Some(device_info);
            }
            InfoPageMsg::PageUpdate(PageUpdate::Stats(device_stats)) => {
                self.device_stats = Some(device_stats);
            }
            InfoPageMsg::ShowTopology => {
                if let (Some(info), Some(topology)) = (&self.device_info, &self.topology) {
                    let gpu_name = info
                        .pci_info
                        .as_ref()
                        .and_then(|pci_info| pci_info.device_pci_info.model.clone())
                        .unwrap_or_else(|| "GPU".to_owned());

                    let mut window = TopologyWindow::builder()
                        .launch((gpu_name, topology.clone()))
                        .detach();
                    window.detach_runtime();
                }
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
