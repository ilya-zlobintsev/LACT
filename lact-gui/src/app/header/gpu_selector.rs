use crate::{CONFIG, app::header::HeaderMsg};
use gtk::glib;
use gtk::prelude::*;
use lact_client::schema::DeviceListEntry;
use lact_schema::DeviceType;
use relm4::{Component, ComponentParts, ComponentSender, WidgetTemplate, css};
use tracing::debug;

#[derive(Debug)]
pub enum GPUSelectorMsg {
    GpuSelected,
}

pub struct GPUSelector {
    devices: Vec<DeviceListEntry>,
    /// `None` when there is at most one GPU: the static row is only in the widget tree under [`Self::Root`].
    dropdown: Option<gtk::DropDown>,
}

#[relm4::component(pub)]
impl Component for GPUSelector {
    type Init = Vec<DeviceListEntry>;
    type Input = GPUSelectorMsg;
    type Output = HeaderMsg;
    type CommandOutput = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            add_css_class: "gpu-selector",
            add_css_class: "gpu-picker-container",
        }
    }

    fn init(
        devices: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let dropdown = if devices.len() <= 1 {
            let item = GpuListItem::init(());
            if let Some(device) = devices.first() {
                item.name_label.set_label(&device.to_string());
                item.id_label.set_label(&device.id);
                item.add_css_class(css::RAISED);
                CONFIG.write().edit(|config| {
                    config.selected_gpu = Some(device.id.clone());
                });
                sender
                    .output(HeaderMsg::GpuSelected(0))
                    .expect("GPU selector output channel closed");
            } else {
                CONFIG.write().edit(|config| {
                    config.selected_gpu = None;
                });
            }
            root.append(item.as_ref());
            // Row stays alive: `root` holds a strong ref after `append`. Dropping `item` only drops the Rust handle.
            None
        } else {
            let string_list = gtk::StringList::new(&[]);
            for device in &devices {
                string_list.append(&device.to_string());
            }

            let selected_index = CONFIG
                .read()
                .selected_gpu
                .as_ref()
                .and_then(|selected_gpu_id| {
                    devices
                        .iter()
                        .position(|d| d.id == *selected_gpu_id)
                        .inspect(|_| debug!("selecting gpu id {selected_gpu_id}"))
                })
                .or_else(|| {
                    devices
                        .iter()
                        .position(|d| d.device_type == DeviceType::Dedicated)
                        .inspect(|i| debug!("selecting default dedicated gpu {}", devices[*i].id))
                })
                .unwrap_or(0) as u32;

            let item_factory = gtk::SignalListItemFactory::new();
            item_factory.connect_setup(|_, list_item| {
                let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();
                let template = GpuListItem::init(());
                list_item.set_child(Some(template.as_ref()));
                unsafe {
                    list_item.set_data("template", template);
                }
            });
            item_factory.connect_bind(glib::clone!(
                #[strong]
                devices,
                move |_, list_item| {
                    let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();
                    if let Some(device) = devices.get(list_item.position() as usize) {
                        unsafe {
                            if let Some(template) = list_item.data::<GpuListItem>("template") {
                                let template = template.as_ref();
                                template.name_label.set_label(&device.to_string());
                                template.id_label.set_label(&device.id);
                            }
                        }
                    }
                }
            ));

            let dropdown = gtk::DropDown::builder()
                .model(&string_list)
                .selected(selected_index)
                .build();
            dropdown.set_factory(Some(&item_factory));
            dropdown.set_list_factory(Some(&item_factory));
            dropdown.set_hexpand(true);
            dropdown.add_css_class("gpu-picker-surface");
            dropdown.connect_selected_notify(move |_| {
                sender.input(GPUSelectorMsg::GpuSelected);
            });
            root.append(&dropdown);
            Some(dropdown)
        };

        let model = GPUSelector { devices, dropdown };
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            GPUSelectorMsg::GpuSelected => {
                let Some(dropdown) = &self.dropdown else {
                    return;
                };
                let selected = dropdown.selected();
                let id = self.devices.get(selected as usize).map(|d| d.id.clone());
                CONFIG.write().edit(|config| {
                    config.selected_gpu = id;
                });
                sender
                    .output(HeaderMsg::GpuSelected(selected))
                    .expect("GPU selector output channel closed");
            }
        }
    }
}

#[relm4::widget_template]
impl WidgetTemplate for GpuListItem {
    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_hexpand: true,

            #[name = "name_label"]
            gtk::Label {
                set_hexpand: true,
                set_halign: gtk::Align::Start,
                set_xalign: 0.0,
                set_ellipsize: gtk::pango::EllipsizeMode::End,
            },

            #[name = "id_label"]
            gtk::Label {
                set_hexpand: true,
                set_halign: gtk::Align::Start,
                set_xalign: 0.0,
                add_css_class: css::DIM_LABEL,
                add_css_class: css::CAPTION,
            },
        }
    }
}
