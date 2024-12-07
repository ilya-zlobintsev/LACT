mod new_profile_dialog;
mod profile_row;

use super::{AppMsg, DebugSnapshot, DisableOverdrive, DumpVBios, ResetConfig, ShowGraphsWindow};
use glib::{clone, SignalHandlerId};
use gtk::prelude::*;
use gtk::*;
use lact_client::schema::DeviceListEntry;
use lact_schema::ProfilesInfo;
use new_profile_dialog::NewProfileDialog;
use profile_row::{ProfileRow, ProfileRowOutput};
use relm4::{
    factory::{DynamicIndex, FactoryVecDeque},
    typed_view::list::{RelmListItem, TypedListView},
    Component, ComponentController, ComponentParts, ComponentSender, RelmWidgetExt,
};
use tracing::trace;

pub struct Header {
    profiles_info: ProfilesInfo,
    gpu_selector: TypedListView<GpuListItem, gtk::SingleSelection>,
    profile_selector: FactoryVecDeque<ProfileRow>,
    selector_label: String,
    select_profile_signal_handler: SignalHandlerId,
    stack: Option<Stack>,
}

#[derive(Debug)]
pub enum HeaderMsg {
    Stack(Stack),
    Profiles(ProfilesInfo),
    AutoProfileSwitch(bool),
    SelectProfile,
    SelectGpu,
    CreateProfile,
}

#[relm4::component(pub)]
impl Component for Header {
    type Init = Vec<DeviceListEntry>;
    type Input = HeaderMsg;
    type Output = AppMsg;
    type CommandOutput = ();

    view! {
        gtk::HeaderBar {
            set_show_title_buttons: true,

            #[wrap(Some)]
            set_title_widget = &StackSwitcher {
                #[watch]
                set_stack: model.stack.as_ref(),
            },

            #[name = "menu_button"]
            pack_start = &gtk::MenuButton {
                #[watch]
                set_label: &model.selector_label,
                #[wrap(Some)]
                set_popover = &gtk::Popover {
                    set_margin_all: 5,
                    set_autohide: false,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 5,

                        gtk::Frame {
                            set_label: Some("GPU"),
                            set_label_align: 0.05,
                            set_margin_all: 5,

                            gtk::ScrolledWindow {
                                set_policy: (gtk::PolicyType::Never, gtk::PolicyType::Automatic),
                                set_propagate_natural_height: true,

                                #[local_ref]
                                gpu_selector -> gtk::ListView { }
                            }
                        },

                        gtk::Frame {
                            set_label: Some("Settings Profile"),
                            set_label_align: 0.05,
                            set_margin_all: 5,

                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_spacing: 5,

                                gtk::CheckButton {
                                    set_label: Some("Switch automatically"),
                                    set_margin_horizontal: 5,
                                    #[watch]
                                    #[block_signal(toggle_auto_profile_handler)]
                                    set_active: model.profiles_info.auto_switch,
                                    connect_toggled[sender] => move |button| {
                                        sender.input(HeaderMsg::AutoProfileSwitch(button.is_active()));
                                    } @ toggle_auto_profile_handler
                                },

                                gtk::ScrolledWindow {
                                    set_policy: (gtk::PolicyType::Never, gtk::PolicyType::Automatic),
                                    set_propagate_natural_height: true,

                                    #[local_ref]
                                    profile_selector -> gtk::ListBox {
                                        set_selection_mode: gtk::SelectionMode::Single,
                                    }
                                },

                                gtk::Box {
                                    set_orientation: gtk::Orientation::Horizontal,
                                    set_spacing: 5,

                                    gtk::Button {
                                        set_expand: true,
                                        set_icon_name: "list-add",
                                        connect_clicked => HeaderMsg::CreateProfile,
                                    },
                                },
                            }
                        },
                    }
                },
            },

            pack_end = &gtk::MenuButton {
                set_icon_name: "open-menu-symbolic",
                set_menu_model: Some(&app_menu),
            }
        },

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
        variants: Self::Init,
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

        let profile_selector = FactoryVecDeque::<ProfileRow>::builder()
            .launch_default()
            .forward(sender.output_sender(), |msg| match msg {
                ProfileRowOutput::MoveUp(profile, index) => move_profile_msg(profile, index, -1),
                ProfileRowOutput::MoveDown(profile, index) => move_profile_msg(profile, index, 1),
                ProfileRowOutput::Delete(profile) => AppMsg::DeleteProfile(profile),
            });
        let select_profile_signal_handler = profile_selector.widget().connect_row_selected(clone!(
            #[strong]
            sender,
            move |_, _| {
                trace!("Profile signal fired");
                let _ = sender.input_sender().send(HeaderMsg::SelectProfile);
            }
        ));

        let model = Self {
            gpu_selector,
            profile_selector,
            selector_label: String::new(),
            profiles_info: ProfilesInfo::default(),
            select_profile_signal_handler,
            stack: None,
        };

        let gpu_selector = &model.gpu_selector.view;
        let profile_selector = model.profile_selector.widget();
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

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            HeaderMsg::Profiles(profiles_info) => self.set_profiles_info(profiles_info),
            HeaderMsg::Stack(stack) => {
                self.stack = Some(stack);
            }
            HeaderMsg::SelectGpu => sender.output(AppMsg::ReloadData { full: true }).unwrap(),
            HeaderMsg::AutoProfileSwitch(auto_switch) => {
                let msg = AppMsg::SelectProfile {
                    profile: self
                        .selected_profile()
                        .filter(|_| !auto_switch)
                        .map(str::to_owned),
                    auto_switch,
                };
                sender.output(msg).unwrap();
            }
            HeaderMsg::SelectProfile => {
                let profile = self.selected_profile();

                if self.profiles_info.current_profile.as_deref() != profile {
                    sender
                        .output(AppMsg::SelectProfile {
                            profile: profile.map(str::to_owned),
                            auto_switch: false,
                        })
                        .unwrap();
                } else {
                    trace!("ignoring profile select event");
                }
            }
            HeaderMsg::CreateProfile => {
                let mut diag_controller = NewProfileDialog::builder()
                    .launch(self.custom_profiles())
                    .forward(sender.output_sender(), |(name, base)| {
                        AppMsg::CreateProfile(name, base)
                    });
                diag_controller.detach_runtime();
            }
        }
        self.update_label();

        self.update_view(widgets, sender);
    }
}

impl Header {
    fn set_profiles_info(&mut self, profiles_info: ProfilesInfo) {
        if self.profiles_info == profiles_info {
            return;
        }

        // self.profile_selector
        //     .widget()
        //     .block_signal(&self.select_profile_signal_handler);

        let mut profiles = self.profile_selector.guard();
        profiles.clear();

        let last = profiles_info.profiles.len().saturating_sub(1);
        for (i, name) in profiles_info.profiles.iter().enumerate() {
            profiles.push_back(ProfileRow::Profile {
                name: name.to_string(),
                first: i == 0,
                last: i == last,
            });
        }
        profiles.push_back(ProfileRow::Default);

        let selected_profile_index = profiles_info.current_profile.as_ref().map(|profile| {
            profiles_info
                .profiles
                .iter()
                .position(|value| value == profile)
                .expect("Active profile is not in the list")
        });

        let new_selected_index = selected_profile_index.unwrap_or_else(|| profiles.len() - 1);
        drop(profiles);

        let new_selected_row = self
            .profile_selector
            .widget()
            .row_at_index(new_selected_index as i32)
            .unwrap();

        self.profile_selector
            .widget()
            .select_row(Some(&new_selected_row));

        // trace!("unblocking signal");
        // self.profile_selector
        //     .widget()
        //     .unblock_signal(&self.select_profile_signal_handler);

        self.profiles_info = profiles_info;
    }

    pub fn selected_gpu_id(&self) -> Option<String> {
        let selected = self.gpu_selector.selection_model.selected();
        self.gpu_selector
            .get(selected)
            .as_ref()
            .map(|item| item.borrow().0.id.clone())
    }

    pub fn auto_switch_profiles(&self) -> bool {
        self.profiles_info.auto_switch
    }

    fn custom_profiles(&self) -> Vec<String> {
        let mut profiles = Vec::with_capacity(self.profile_selector.len());
        for i in 0..self.profile_selector.len() {
            let item = self.profile_selector.get(i).unwrap();
            if let ProfileRow::Profile { name, .. } = item {
                profiles.push(name.clone());
            }
        }
        profiles
    }

    fn selected_profile(&self) -> Option<&str> {
        self.profile_selector
            .widget()
            .selected_row()
            .and_then(|row| self.profile_selector.get(row.index() as usize))
            .and_then(|item| match item {
                ProfileRow::Default => None,
                ProfileRow::Profile { name, .. } => Some(name.as_str()),
            })
    }

    fn update_label(&mut self) {
        let gpu_index = self.gpu_selector.selection_model.selected();
        let profile = self.selected_profile().unwrap_or("Default");

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

fn move_profile_msg(profile: ProfileRow, index: DynamicIndex, offset: i64) -> AppMsg {
    let name = profile.name().expect("Default profile cannot be moved");
    let new_index = (index.current_index() as i64).saturating_add(offset);
    AppMsg::MoveProfile(name, new_index as usize)
}
