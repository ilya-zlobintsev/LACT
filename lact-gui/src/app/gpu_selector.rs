use gtk::glib;
use gtk::prelude::*;
use lact_client::schema::DeviceListEntry;
use relm4::{ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent, WidgetTemplate, css};

pub struct GpuSelector {
    devices: Vec<DeviceListEntry>,
}

#[relm4::component(pub)]
impl SimpleComponent for GpuSelector {
    type Init = (Vec<DeviceListEntry>, Option<String>);
    type Input = ();
    type Output = String;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            add_css_class: "sidebar-section",
            add_css_class: "gpu-selector",

            gtk::Label {
                set_label: "GPU",
                set_halign: gtk::Align::Start,
                set_margin_horizontal: 4,
                add_css_class: css::DIM_LABEL,
                add_css_class: css::CAPTION,
            },

            #[name = "dropdown"]
            gtk::DropDown {
                set_hexpand: true,
            },
        }
    }

    fn init(
        (devices, selected_gpu_id): Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self { devices };
        let widgets = view_output!();

        let (button_factory, list_factory, string_list) = Self::build_factories(&model.devices);
        widgets.dropdown.set_model(Some(&string_list));
        widgets.dropdown.set_factory(Some(&button_factory));
        widgets.dropdown.set_list_factory(Some(&list_factory));
        if let Some(selected_gpu_id) = selected_gpu_id
            && let Some(index) = model
                .devices
                .iter()
                .position(|device| device.id == selected_gpu_id)
        {
            widgets.dropdown.set_selected(index as u32);
        }

        let devices_vec = model.devices.clone();
        let sender_clone = sender.clone();
        widgets.dropdown.connect_selected_notify(move |dropdown| {
            Self::select_gpu(&devices_vec, dropdown.selected(), &sender_clone);
        });

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
    ) {
        let devices_vec = devices.to_vec();
        let string_list = gtk::StringList::new(&[]);
        for device in &devices_vec {
            string_list.append(&device.to_string());
        }

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

        (button_factory, list_factory, string_list)
    }

    fn select_gpu(devices: &[DeviceListEntry], selected: u32, sender: &ComponentSender<Self>) {
        if let Some(device) = devices.get(selected as usize) {
            sender.output(device.id.clone()).unwrap();
        }
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
