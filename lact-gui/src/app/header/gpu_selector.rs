use crate::{
    CONFIG,
    app::{APP_BROKER, msg::AppMsg},
};
use gtk::glib::clone;
use gtk::prelude::*;
use lact_client::schema::DeviceListEntry;
use lact_schema::DeviceType;
use relm4::{
    Component, ComponentParts, ComponentSender, css,
    typed_view::list::{RelmListItem, TypedListView},
};
use tracing::debug;

#[derive(Debug)]
pub enum GPUSelectorMsg {
    GpuSelected,
}

pub struct GPUSelector {
    view: TypedListView<GpuListItem, gtk::SingleSelection>,
}

#[relm4::component(pub)]
impl Component for GPUSelector {
    type Init = Vec<DeviceListEntry>;
    type Input = GPUSelectorMsg;
    type Output = ();
    type CommandOutput = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 5,

            gtk::Label {
                set_label: "GPU",
                add_css_class: css::HEADING,
            },

            gtk::ScrolledWindow {
                add_css_class: "gpu-picker-container",
                set_policy: (gtk::PolicyType::Never, gtk::PolicyType::Automatic),
                set_propagate_natural_height: true,

                #[local_ref]
                gpu_list -> gtk::ListView {}
            },
        }
    }

    fn init(
        variants: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut view = TypedListView::<GpuListItem, gtk::SingleSelection>::new();
        view.extend_from_iter(variants.into_iter().map(GpuListItem));

        view.selection_model.connect_selection_changed(clone!(
            #[strong]
            sender,
            move |_, _, _| {
                sender.input(GPUSelectorMsg::GpuSelected);
            }
        ));

        let idx = CONFIG
            .read()
            .selected_gpu
            .as_ref()
            .and_then(|selected_gpu_id| {
                for idx in 0..view.len() {
                    let gpu_item = view.get(idx).unwrap();
                    if gpu_item.borrow().0.id == *selected_gpu_id {
                        debug!("selecting gpu id {selected_gpu_id}");
                        return Some(idx);
                    }
                }
                None
            })
            .or_else(|| {
                for idx in 0..view.len() {
                    let gpu_item = view.get(idx).unwrap();
                    if gpu_item.borrow().0.device_type == DeviceType::Dedicated {
                        debug!("selecting default dedicated gpu {}", gpu_item.borrow().0.id);
                        return Some(idx);
                    }
                }
                None
            });

        if let Some(idx) = idx {
            view.selection_model.set_selected(idx);
        }

        sender.input(GPUSelectorMsg::GpuSelected);

        let model = GPUSelector { view };
        let gpu_list = &model.view.view;
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            GPUSelectorMsg::GpuSelected => {
                let selected = self.view.selection_model.selected();
                let id = self
                    .view
                    .get(selected)
                    .as_ref()
                    .map(|item| item.borrow().0.id.clone());
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
        self.view.selection_model.selected()
    }
}

struct GpuListItem(DeviceListEntry);

struct GpuListItemWidgets {
    name_label: gtk::Label,
    id_label: gtk::Label,
    type_label: gtk::Label,
}

impl RelmListItem for GpuListItem {
    type Root = gtk::Box;
    type Widgets = GpuListItemWidgets;

    fn setup(_list_item: &gtk::ListItem) -> (Self::Root, Self::Widgets) {
        relm4::view! {
            root = gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                #[name = "name_label"]
                gtk::Label,

                gtk::Box {
                    set_spacing: 5,
                    set_orientation: gtk::Orientation::Horizontal,
                },

                #[name = "id_label"]
                gtk::Label {
                    add_css_class: "subtitle",
                },

                #[name = "type_label"]
                gtk::Label {
                    add_css_class: "subtitle",
                },
            }
        };

        let widgets = GpuListItemWidgets {
            name_label,
            id_label,
            type_label,
        };
        (root, widgets)
    }

    fn bind(&mut self, widgets: &mut Self::Widgets, _root: &mut Self::Root) {
        widgets
            .name_label
            .set_label(self.0.name.as_deref().unwrap_or("Unknown"));
        widgets.id_label.set_label(&self.0.id);
        widgets
            .type_label
            .set_label(&self.0.device_type.to_string());
    }
}
