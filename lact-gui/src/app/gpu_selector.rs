use crate::CONFIG;
use gtk::glib;
use gtk::prelude::*;
use lact_client::schema::DeviceListEntry;
use lact_schema::DeviceType;
use relm4::{ComponentParts, ComponentSender, SimpleComponent, WidgetTemplate, css};
use tracing::debug;

pub struct GpuSelector {
    devices: Vec<DeviceListEntry>,
}

#[relm4::component(pub)]
impl SimpleComponent for GpuSelector {
    type Init = Vec<DeviceListEntry>;
    type Input = ();
    type Output = u32;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            add_css_class: "gpu-selector",

            #[name = "dropdown"]
            gtk::DropDown {
                set_hexpand: true,
            },
        }
    }

    fn init(
        devices: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let use_dropdown = devices.len() > 1;

        let model = Self { devices };
        let widgets = view_output!();

        if use_dropdown {
            let (button_factory, list_factory, string_list, selected_index) =
                Self::build_factories(&model.devices);
            widgets.dropdown.set_model(Some(&string_list));
            widgets.dropdown.set_factory(Some(&button_factory));
            widgets.dropdown.set_list_factory(Some(&list_factory));
            widgets.dropdown.set_selected(selected_index);

            Self::select_gpu(&model.devices, selected_index, &sender);

            let devices_vec = model.devices.clone();
            let sender_clone = sender.clone();
            widgets.dropdown.connect_selected_notify(move |dropdown| {
                Self::select_gpu(&devices_vec, dropdown.selected(), &sender_clone);
            });
        } else {
            Self::select_gpu(&model.devices, 0, &sender);
        }

        ComponentParts { model, widgets }
    }
}

impl GpuSelector {
    fn build_factories(
        devices: &[DeviceListEntry],
    ) -> (
        gtk::SignalListItemFactory,
        gtk::SignalListItemFactory,
        gtk::StringList,
        u32,
    ) {
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

        let button_factory = gtk::SignalListItemFactory::new();
        button_factory.connect_setup(|_, list_item| {
            let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();
            let label = gtk::Label::builder()
                .hexpand(true)
                .halign(gtk::Align::Start)
                .ellipsize(gtk::pango::EllipsizeMode::End)
                .build();
            list_item.set_child(Some(&label));
        });
        button_factory.connect_bind(glib::clone!(
            #[strong]
            devices_vec,
            move |_, list_item| {
                let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();
                if let Some(device) = devices_vec.get(list_item.position() as usize) {
                    let label = list_item.child().unwrap().downcast::<gtk::Label>().unwrap();
                    label.set_label(&device.to_string());
                }
            }
        ));

        // Detailed factory for the popup list — shows name, ID, and type
        let list_factory = gtk::SignalListItemFactory::new();
        list_factory.connect_setup(|_, list_item| {
            let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();
            let template = GpuListItem::init(());
            list_item.set_child(Some(template.as_ref()));
        });
        list_factory.connect_bind(glib::clone!(
            #[strong]
            devices_vec,
            move |_, list_item| {
                let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();
                if let Some(device) = devices_vec.get(list_item.position() as usize) {
                    let child = list_item.child().unwrap();
                    let name_label = child
                        .first_child()
                        .unwrap()
                        .downcast::<gtk::Label>()
                        .unwrap();
                    let id_label = name_label
                        .next_sibling()
                        .unwrap()
                        .downcast::<gtk::Label>()
                        .unwrap();
                    let type_label = id_label
                        .next_sibling()
                        .unwrap()
                        .downcast::<gtk::Label>()
                        .unwrap();
                    name_label.set_label(&device.to_string());
                    id_label.set_label(&device.id);
                    type_label.set_markup(&format!("<b>{}</b>", device.device_type));
                }
            }
        ));

        (button_factory, list_factory, string_list, selected_index)
    }

    fn select_gpu(devices: &[DeviceListEntry], selected: u32, sender: &ComponentSender<Self>) {
        let id = devices
            .get(selected as usize)
            .map(|device| device.id.clone());
        CONFIG.write().edit(|config| {
            config.selected_gpu = id;
        });
        sender.output(selected).unwrap()
    }
}

#[relm4::widget_template(pub)]
impl WidgetTemplate for GpuListItem {
    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_hexpand: true,

            gtk::Label {
                set_hexpand: true,
                set_halign: gtk::Align::Center,
                set_ellipsize: gtk::pango::EllipsizeMode::End,
            },

            gtk::Label {
                set_hexpand: true,
                set_halign: gtk::Align::Center,
                add_css_class: css::DIM_LABEL,
                add_css_class: css::CAPTION,
            },

            gtk::Label {
                set_hexpand: true,
                set_halign: gtk::Align::Center,
                add_css_class: css::DIM_LABEL,
                add_css_class: css::CAPTION,
            },
        }
    }
}
