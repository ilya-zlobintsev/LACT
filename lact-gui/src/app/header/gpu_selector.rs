use crate::{CONFIG, app::header::HeaderMsg};
use adw::prelude::*;
use gtk::glib;
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
    combo_row: adw::ComboRow,
}

#[relm4::component(pub)]
impl Component for GPUSelector {
    type Init = Vec<DeviceListEntry>;
    type Input = GPUSelectorMsg;
    type Output = HeaderMsg;
    type CommandOutput = ();

    view! {
        adw::PreferencesGroup {
            add_css_class: "gpu-selector",

            #[local_ref]
            add = combo_row -> adw::ComboRow {
                set_title: "GPU",
                connect_selected_notify[sender] => move |_| {
                    sender.input(GPUSelectorMsg::GpuSelected);
                },
            },
        }
    }

    fn init(
        devices: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
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
        item_factory.connect_setup(|_, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let template = GpuListItem::init(());
            item.set_child(Some(template.as_ref()));
            unsafe {
                item.set_data("template", template);
            }
        });
        item_factory.connect_bind(glib::clone!(
            #[strong]
            devices,
            move |_, item| {
                let item = item.downcast_ref::<gtk::ListItem>().unwrap();
                if let Some(device) = devices.get(item.position() as usize) {
                    unsafe {
                        if let Some(template) = item.data::<GpuListItem>("template") {
                            let template = template.as_ref();
                            template.name_label.set_label(&device.to_string());
                            template.id_label.set_label(&device.id);
                        }
                    }
                }
            }
        ));

        let combo_row = adw::ComboRow::new();
        combo_row.set_model(Some(&string_list));
        combo_row.set_factory(Some(&item_factory));
        combo_row.set_selected(selected_index);

        // part of the application startup, reloads the data which cleans global set_sensetive: false
        // might be good to refactor
        let _ = sender.output(HeaderMsg::GpuSelected(selected_index));

        let model = GPUSelector { devices, combo_row };
        let combo_row = &model.combo_row;
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            GPUSelectorMsg::GpuSelected => {
                let selected = self.combo_row.selected();
                let id = self.devices.get(selected as usize).map(|d| d.id.clone());
                CONFIG.write().edit(|config| {
                    config.selected_gpu = id;
                });
                let _ = sender.output(HeaderMsg::GpuSelected(selected));
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
            set_cursor_from_name: Some("pointer"),

            #[name = "name_label"]
            gtk::Label {
                set_hexpand: true,
                set_halign: gtk::Align::Fill,
                set_xalign: 0.0,
                set_ellipsize: gtk::pango::EllipsizeMode::End,
            },

            #[name = "id_label"]
            gtk::Label {
                set_hexpand: true,
                set_halign: gtk::Align::Fill,
                set_xalign: 0.0,
                add_css_class: css::DIM_LABEL,
                add_css_class: css::CAPTION,
            },
        }
    }
}
