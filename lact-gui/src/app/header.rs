use std::fmt;

use super::{AppMsg, DebugSnapshot, DisableOverdrive, DumpVBios, ResetConfig, ShowGraphsWindow};
use glib::clone;
use gtk::prelude::*;
use gtk::*;
use lact_client::schema::DeviceListEntry;
use lact_schema::ProfilesInfo;
use relm4::{
    typed_view::list::{RelmListItem, TypedListView},
    ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent,
};

pub struct Header {
    gpu_selector: TypedListView<GpuListItem, gtk::SingleSelection>,
    profile_selector: TypedListView<ProfileListItem, gtk::SingleSelection>,
    selector_label: String,
}

#[derive(Debug)]
pub enum HeaderMsg {
    Profiles(ProfilesInfo),
    SelectProfile,
    SelectGpu,
}

#[relm4::component(pub)]
impl SimpleComponent for Header {
    type Init = (Vec<DeviceListEntry>, gtk::Stack);
    type Input = HeaderMsg;
    type Output = AppMsg;

    view! {
        gtk::HeaderBar {
            set_show_title_buttons: true,

            #[wrap(Some)]
            set_title_widget = &StackSwitcher {
                set_stack: Some(&stack),
            },

            #[name = "menu_button"]
            pack_start = &gtk::MenuButton {
                #[watch]
                set_label: &model.selector_label,
                #[wrap(Some)]
                set_popover = &gtk::Popover {
                    set_margin_all: 5,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 5,

                        gtk::Frame {
                            set_label: Some("GPU"),

                            gtk::ScrolledWindow {
                                set_policy: (gtk::PolicyType::Never, gtk::PolicyType::Automatic),
                                set_propagate_natural_height: true,

                                #[local_ref]
                                gpu_selector -> gtk::ListView { }
                            }
                        },

                        gtk::Frame {
                            set_label: Some("Settings Profile"),

                            gtk::ScrolledWindow {
                                set_policy: (gtk::PolicyType::Never, gtk::PolicyType::Automatic),
                                set_propagate_natural_height: true,

                                #[local_ref]
                                profile_selector -> gtk::ListView { }
                            }
                        },
                    }
                },
            },

            pack_end = &gtk::MenuButton {
                set_icon_name: "open-menu-symbolic",
                set_menu_model: Some(&app_menu),
            }
        }
    }

    menu! {
        app_menu: {
            section! {
                "Show historical charts" => ShowGraphsWindow,
            },
            section! {
                "Generate debug snapshot" => DebugSnapshot,
                "Dump VBIOS" => DumpVBios,
            } ,
            section! {
                "Disable overclocking support" => DisableOverdrive,
                "Reset all configuration" => ResetConfig,
            }
        }
    }

    fn init(
        (variants, stack): Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        sender.input(HeaderMsg::SelectGpu);

        let mut gpu_selector = TypedListView::<_, gtk::SingleSelection>::new();
        gpu_selector.extend_from_iter(variants.into_iter().map(GpuListItem));

        gpu_selector
            .selection_model
            .connect_selection_changed(clone!(
                #[strong]
                sender,
                move |_, _, _| {
                    sender.input(HeaderMsg::SelectGpu);
                }
            ));

        let profile_selector = TypedListView::<_, gtk::SingleSelection>::new();
        profile_selector
            .selection_model
            .connect_selection_changed(move |_, _, _| {
                sender.input(HeaderMsg::SelectProfile);
            });

        let model = Self {
            gpu_selector,
            profile_selector,
            selector_label: String::new(),
        };

        let gpu_selector = &model.gpu_selector.view;
        let profile_selector = &model.profile_selector.view;
        let widgets = view_output!();

        widgets.menu_button.connect_label_notify(|menu_button| {
            let label_box = menu_button
                .first_child()
                .unwrap()
                .downcast::<gtk::ToggleButton>()
                .unwrap()
                .child()
                .unwrap()
                .downcast::<gtk::Box>()
                .unwrap();
            // Limits the length of text in the menu button
            let selector_label = label_box
                .first_child()
                .unwrap()
                .downcast::<Label>()
                .unwrap();
            selector_label.set_ellipsize(pango::EllipsizeMode::End);
            selector_label.set_width_chars(14);
        });

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            HeaderMsg::Profiles(profiles_info) => {
                let selected_index = match &profiles_info.current_profile {
                    Some(profile) => profiles_info
                        .profiles
                        .iter()
                        .position(|value| value == profile)
                        .expect("Active profile is not in the list"),
                    None => 0,
                };

                self.profile_selector.clear();
                self.profile_selector.append(ProfileListItem::Default);

                for profile in profiles_info.profiles {
                    self.profile_selector
                        .append(ProfileListItem::Profile(profile));
                }

                self.profile_selector
                    .selection_model
                    .set_selected(selected_index as u32);
            }
            HeaderMsg::SelectGpu => sender.output(AppMsg::ReloadData { full: true }).unwrap(),
            HeaderMsg::SelectProfile => {
                let selected_index = self.profile_selector.selection_model.selected();
                let item = self
                    .profile_selector
                    .get(selected_index)
                    .expect("Invalid item selected");
                let selected_profile = match &*item.borrow() {
                    ProfileListItem::Default => None,
                    ProfileListItem::Profile(name) => Some(name.clone()),
                };
                sender
                    .output(AppMsg::SelectProfile(selected_profile))
                    .unwrap();
            }
        }
        self.update_label();
    }
}

impl Header {
    pub fn selected_gpu_id(&self) -> Option<String> {
        let selected = self.gpu_selector.selection_model.selected();
        self.gpu_selector
            .get(selected)
            .as_ref()
            .map(|item| item.borrow().0.id.clone())
    }

    fn update_label(&mut self) {
        let gpu_index = self.gpu_selector.selection_model.selected();
        let profile = self
            .profile_selector
            .get(self.profile_selector.selection_model.selected())
            .as_ref()
            .map(|item| item.borrow().to_string())
            .unwrap_or_else(|| "<Unknown>".to_owned());

        self.selector_label = format!("GPU {gpu_index} | {profile}");
    }
}

struct GpuListItem(DeviceListEntry);

impl RelmListItem for GpuListItem {
    type Root = gtk::Label;
    type Widgets = gtk::Label;

    fn setup(_list_item: &gtk::ListItem) -> (Self::Root, Self::Widgets) {
        let label = gtk::Label::new(None);
        label.set_margin_all(5);
        (label.clone(), label)
    }

    fn bind(&mut self, widgets: &mut Self::Widgets, _root: &mut Self::Root) {
        widgets.set_label(self.0.name.as_deref().unwrap_or(&self.0.id));
    }
}

enum ProfileListItem {
    Default,
    Profile(String),
}

impl fmt::Display for ProfileListItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            ProfileListItem::Default => "Default",
            ProfileListItem::Profile(name) => name,
        };
        text.fmt(f)
    }
}

impl RelmListItem for ProfileListItem {
    type Root = Label;
    type Widgets = Label;

    fn setup(_list_item: &gtk::ListItem) -> (Self::Root, Self::Widgets) {
        let label = gtk::Label::new(None);
        label.set_margin_all(5);
        (label.clone(), label)
    }

    fn bind(&mut self, widgets: &mut Self::Widgets, _root: &mut Self::Root) {
        widgets.set_label(&self.to_string());
    }
}
