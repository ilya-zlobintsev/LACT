use crate::{
    CONFIG,
    app::{APP_BROKER, msg::AppMsg},
};
use adw::prelude::*;
use lact_client::schema::DeviceListEntry;
use lact_schema::DeviceType;
use relm4::{css, Component, ComponentParts, ComponentSender};
use tracing::debug;

#[derive(Debug)]
pub enum GPUSelectorMsg {
    GpuSelected,
}

pub struct GPUSelector {
    devices: Vec<DeviceListEntry>,
    selected_index: u32,
}

#[relm4::component(pub)]
impl Component for GPUSelector {
    type Init = Vec<DeviceListEntry>;
    type Input = GPUSelectorMsg;
    type Output = ();
    type CommandOutput = ();

    view! {
        adw::ComboRow {
            set_title: "GPU",
            set_css_classes: &[css::CARD, "gpu-selector"],
            set_hexpand: true,

            connect_selected_notify[sender] => move |_| {
                sender.input(GPUSelectorMsg::GpuSelected);
            },
        },
    }

    fn init(
        variants: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let string_list = gtk::StringList::new(&[]);
        for device in &variants {
            string_list.append(&device.to_string());
        }
        root.set_model(Some(&string_list));

        let selected_index = CONFIG
            .read()
            .selected_gpu
            .as_ref()
            .and_then(|selected_gpu_id| {
                variants.iter().position(|d| {
                    if d.id == *selected_gpu_id {
                        debug!("selecting gpu id {selected_gpu_id}");
                        true
                    } else {
                        false
                    }
                })
            })
            .or_else(|| {
                variants.iter().position(|d| {
                    if d.device_type == DeviceType::Dedicated {
                        debug!("selecting default dedicated gpu {}", d.id);
                        true
                    } else {
                        false
                    }
                })
            })
            .unwrap_or(0) as u32;

        root.set_selected(selected_index);

        sender.input(GPUSelectorMsg::GpuSelected);

        let model = GPUSelector {
            devices: variants,
            selected_index,
        };
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>, root: &Self::Root) {
        match message {
            GPUSelectorMsg::GpuSelected => {
                self.selected_index = root.selected();
                let id = self
                    .devices
                    .get(self.selected_index as usize)
                    .map(|d| d.id.clone());
                CONFIG.write().edit(|config| {
                    config.selected_gpu = id;
                });
                APP_BROKER.send(AppMsg::ReloadData { full: true });
            }
        }
    }
}

impl GPUSelector {
    pub fn selected_index(&self) -> u32 {
        self.selected_index
    }
}
