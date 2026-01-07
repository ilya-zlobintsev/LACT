use gtk::{
    glib::GString,
    prelude::{EditableExt, GtkWindowExt, OrientableExt, WidgetExt},
    NoSelection,
};
use relm4::{
    typed_view::list::{RelmListItem, TypedListView},
    view, ComponentParts, ComponentSender, SimpleComponent,
};

pub struct VulkanFeaturesWindow {
    features_view: TypedListView<VulkanFeature, NoSelection>,
}

#[derive(Debug)]
pub enum AppMsg {
    FilterChanged(GString),
}

#[relm4::component(pub)]
impl SimpleComponent for VulkanFeaturesWindow {
    type Init = (Vec<VulkanFeature>, String);

    type Input = AppMsg;
    type Output = ();

    view! {
        gtk::Window {
            set_title: Some(&title),
            set_default_width: 500,
            set_default_height: 700,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                #[name = "search_entry"]
                gtk::SearchEntry {
                    connect_search_changed[sender] => move |entry| {
                        sender.input(AppMsg::FilterChanged(entry.text()));
                    },

                    connect_stop_search[root] => move |_| {
                        root.close();
                    },
                },

                gtk::ScrolledWindow {
                    #[local_ref]
                    features_list -> gtk::ListView {
                        set_show_separators: true,
                    },

                    set_vexpand: true,
                }
            },


            add_controller = gtk::ShortcutController {
                set_scope: gtk::ShortcutScope::Global,

                add_shortcut = gtk::Shortcut {
                  set_trigger: Some(gtk::ShortcutTrigger::parse_string("Escape|<Ctrl>w").unwrap()),
                  set_action: Some(gtk::ShortcutAction::parse_string("action(window.close)").unwrap()),
                }
            }
        }
    }

    fn init(
        (features, title): Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut features_view: TypedListView<VulkanFeature, NoSelection> = TypedListView::new();
        features_view.extend_from_iter(features);

        let mut model = VulkanFeaturesWindow { features_view };

        let features_list = &model.features_view.view;

        let widgets = view_output!();

        model.features_view.add_filter({
            let search_entry = widgets.search_entry.clone();
            move |feature| {
                feature
                    .name
                    .to_lowercase()
                    .contains(&search_entry.text().to_lowercase())
            }
        });

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            AppMsg::FilterChanged(filter) => {
                self.features_view.set_filter_status(0, false);
                if !filter.is_empty() {
                    self.features_view.set_filter_status(0, true);
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct VulkanFeature {
    pub name: String,
    pub supported: bool,
}

pub struct VulkanFeatureWidgets {
    label: gtk::Label,
    image: gtk::Image,
}

impl RelmListItem for VulkanFeature {
    type Root = gtk::Box;
    type Widgets = VulkanFeatureWidgets;

    fn setup(_: &gtk::ListItem) -> (Self::Root, Self::Widgets) {
        view! {
            root_box = gtk::Box {
                set_focus_on_click: true,
                set_hexpand: true,
                set_hexpand_set: true,
                set_margin_start: 20,
                set_margin_end: 20,
                set_margin_top: 10,
                set_margin_bottom: 10,

                #[name = "label"]
                gtk::Label {
                    set_halign: gtk::Align::Start,
                    set_hexpand: true,
                    set_selectable: true,
                },

                #[name = "image"]
                gtk::Image {
                    set_halign: gtk::Align::End,
                },
            }
        }

        let widgets = VulkanFeatureWidgets { label, image };
        (root_box, widgets)
    }

    fn bind(&mut self, widgets: &mut Self::Widgets, _root: &mut Self::Root) {
        widgets.label.set_label(&self.name);

        let icon = match self.supported {
            true => "object-select-symbolic",
            false => "action-unavailable-symbolic",
        };
        widgets.image.set_icon_name(Some(icon));
    }
}
