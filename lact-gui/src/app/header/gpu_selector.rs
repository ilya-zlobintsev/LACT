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

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_hexpand: true,

                    gtk::Label {
                        #[watch]
                        set_label: &model.single_gpu_name,
                        set_hexpand: true,
                        set_halign: gtk::Align::Start,
                        set_xalign: 0.0,
                        set_ellipsize: gtk::pango::EllipsizeMode::End,
                    },

                    gtk::Label {
                        #[watch]
                        set_label: &model.single_gpu_id,
                        set_hexpand: true,
                        set_halign: gtk::Align::Start,
                        set_xalign: 0.0,
                        add_css_class: css::DIM_LABEL,
                        add_css_class: css::CAPTION,
                    },
                }
            }
        }
    }

    fn init(
        devices: Self::Init,
        root: Self::Root,
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
            let (dropdown, selected_index) = Self::build_dropdown(&devices, sender.clone());
            Self::select_gpu(&devices, selected_index, &sender);
            root.append(&dropdown);
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
    fn build_dropdown(
        devices: &[DeviceListEntry],
        sender: ComponentSender<Self>,
    ) -> (gtk::DropDown, u32) {
        let devices = devices.to_vec();
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
                    .position(|device| device.id == *selected_gpu_id)
                    .inspect(|_| debug!("selecting gpu id {selected_gpu_id}"))
            })
            .or_else(|| {
                devices
                    .iter()
                    .position(|device| device.device_type == DeviceType::Dedicated)
                    .inspect(|index| {
                        debug!("selecting default dedicated gpu {}", devices[*index].id)
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

        dropdown.connect_selected_notify(move |dropdown| {
            Self::select_gpu(&devices, dropdown.selected(), &sender);
        });

        (dropdown, selected_index)
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
