use crate::{CONFIG, app::header::HeaderMsg};
use gtk::glib;
use gtk::prelude::*;
use lact_client::schema::DeviceListEntry;
use lact_schema::DeviceType;
use relm4::{ComponentParts, ComponentSender, SimpleComponent, WidgetTemplate, css};
use tracing::debug;

pub struct GPUSelector {
    multi_gpu: bool,
    single_gpu_name: String,
    single_gpu_id: String,
}

#[relm4::component(pub)]
impl SimpleComponent for GPUSelector {
    type Init = Vec<DeviceListEntry>;
    type Input = ();
    type Output = HeaderMsg;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            add_css_class: "gpu-selector",

            gtk::Button {
                #[watch]
                set_visible: !model.multi_gpu && !model.single_gpu_id.is_empty(),
                set_can_focus: false,
                set_hexpand: true,

                #[template]
                GpuListItem {
                    #[template_child]
                    name_label {
                        #[watch]
                        set_label: &model.single_gpu_name,
                    },

                    #[template_child]
                    id_label {
                        #[watch]
                        set_label: &model.single_gpu_id,
                    },
                }
            },

            #[name = "dropdown"]
            gtk::DropDown {
                #[watch]
                set_visible: model.multi_gpu,
                set_hexpand: true,
            },
        }
    }

    fn init(
        devices: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let multi_gpu = devices.len() > 1;
        let (single_gpu_name, single_gpu_id) = devices
            .first()
            .map(|device| (device.to_string(), device.id.clone()))
            .unwrap_or_default();

        let model = Self {
            multi_gpu,
            single_gpu_name,
            single_gpu_id,
        };
        let widgets = view_output!();

        if multi_gpu {
            let (factory, string_list, selected_index) = Self::build_factory(&devices);
            widgets.dropdown.set_model(Some(&string_list));
            widgets.dropdown.set_factory(Some(&factory));
            widgets.dropdown.set_list_factory(Some(&factory));
            widgets.dropdown.set_selected(selected_index);

            let devices_vec = devices;
            Self::select_gpu(&devices_vec, selected_index, &sender);

            let sender_clone = sender.clone();
            widgets.dropdown.connect_selected_notify(move |dropdown| {
                Self::select_gpu(&devices_vec, dropdown.selected(), &sender_clone);
            });
        } else {
            CONFIG.write().edit(|config| {
                config.selected_gpu = devices.first().map(|device| device.id.clone());
            });

            if !devices.is_empty() {
                sender
                    .output(HeaderMsg::GpuSelected(0))
                    .expect("GPU selector output channel closed");
            }
        }

        ComponentParts { model, widgets }
    }
}

impl GPUSelector {
    fn build_factory(
        devices: &[DeviceListEntry],
    ) -> (gtk::SignalListItemFactory, gtk::StringList, u32) {
        let devices_vec = devices.to_vec();
        let string_list = gtk::StringList::new(&[]);
        for device in &devices_vec {
            string_list.append(&device.to_string());
        }

        let selected_index = CONFIG
            .read()
            .selected_gpu
            .as_ref()
            .and_then(|selected_gpu_id| {
                devices_vec
                    .iter()
                    .position(|device| device.id == *selected_gpu_id)
                    .inspect(|_| debug!("selecting gpu id {selected_gpu_id}"))
            })
            .or_else(|| {
                devices_vec
                    .iter()
                    .position(|device| device.device_type == DeviceType::Dedicated)
                    .inspect(|index| {
                        debug!("selecting default dedicated gpu {}", devices_vec[*index].id)
                    })
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
            devices_vec,
            move |_, list_item| {
                let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();
                if let Some(device) = devices_vec.get(list_item.position() as usize) {
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

        (item_factory, string_list, selected_index)
    }

    fn select_gpu(devices: &[DeviceListEntry], selected: u32, sender: &ComponentSender<Self>) {
        let id = devices
            .get(selected as usize)
            .map(|device| device.id.clone());
        CONFIG.write().edit(|config| {
            config.selected_gpu = id;
        });
        sender
            .output(HeaderMsg::GpuSelected(selected))
            .expect("GPU selector output channel closed");
    }
}

#[relm4::widget_template(pub)]
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
