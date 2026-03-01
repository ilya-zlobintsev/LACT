use crate::{
    CONFIG,
    app::{APP_BROKER, msg::AppMsg},
};
use adw::prelude::*;
use gtk::glib;
use lact_client::schema::DeviceListEntry;
use lact_schema::DeviceType;
use relm4::{Component, ComponentParts, ComponentSender};
use tracing::debug;

#[derive(Debug)]
pub enum GPUSelectorMsg {
    GpuSelected,
}

pub struct GPUSelector {
    devices: Vec<DeviceListEntry>,
    selected_index: u32,
    combo_row: adw::ComboRow,
}

#[relm4::component(pub)]
impl Component for GPUSelector {
    type Init = Vec<DeviceListEntry>;
    type Input = GPUSelectorMsg;
    type Output = ();
    type CommandOutput = ();

    view! {
        adw::PreferencesGroup {
            add_css_class: "gpu-selector",

            #[local_ref]
            combo_row -> adw::ComboRow {
                set_title: "GPU",
                set_cursor_from_name: Some("pointer"),
                connect_selected_notify[sender] => move |_| {
                    sender.input(GPUSelectorMsg::GpuSelected);
                },
            },
        }
    }

    fn init(
        variants: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let string_list = gtk::StringList::new(&[]);
        for device in &variants {
            string_list.append(&device.to_string());
        }

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

        let list_factory = gtk::SignalListItemFactory::new();
        list_factory.connect_setup(|_, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
            let name_label = gtk::Label::new(None);
            let id_label = gtk::Label::new(None);
            id_label.add_css_class("dim-label");
            id_label.add_css_class("caption");
            container.append(&name_label);
            container.append(&id_label);
            container.set_cursor_from_name(Some("pointer"));
            item.set_child(Some(&container));
        });
        list_factory.connect_bind(glib::clone!(
            #[strong]
            variants,
            move |_, item| {
                let item = item.downcast_ref::<gtk::ListItem>().unwrap();
                if let Some(device) = variants.get(item.position() as usize) {
                    let container = item.child().unwrap().downcast::<gtk::Box>().unwrap();
                    let mut child = container.first_child();
                    child
                        .as_ref()
                        .unwrap()
                        .downcast_ref::<gtk::Label>()
                        .unwrap()
                        .set_label(&device.to_string());
                    child = child.unwrap().next_sibling();
                    child
                        .as_ref()
                        .unwrap()
                        .downcast_ref::<gtk::Label>()
                        .unwrap()
                        .set_label(&device.id);
                }
            }
        ));

        let combo_row = adw::ComboRow::new();
        combo_row.set_model(Some(&string_list));
        combo_row.set_list_factory(Some(&list_factory));
        combo_row.set_selected(selected_index);

        sender.input(GPUSelectorMsg::GpuSelected);

        let model = GPUSelector {
            devices: variants,
            selected_index,
            combo_row,
        };
        let combo_row = &model.combo_row;
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            GPUSelectorMsg::GpuSelected => {
                self.selected_index = self.combo_row.selected();
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
