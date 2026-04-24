mod new_profile_dialog;
mod profile_rename_dialog;
mod profile_row;
pub mod profile_rule_window;

use crate::I18N;
use crate::app::msg::AppMsg;
use crate::app::profiles::new_profile_dialog::NewProfileDialog;
use crate::app::profiles::profile_rename_dialog::ProfileRenameDialog;
use crate::app::profiles::profile_row::ProfileRow;
use crate::app::profiles::profile_row::ProfileRowType;
use crate::app::profiles::profile_rule_window::ProfileEditParams;
use crate::app::profiles::profile_rule_window::ProfileRuleWindow;
use gtk::glib::clone;
use gtk::prelude::*;
use i18n_embed_fl::fl;
use lact_schema::ProfilesInfo;
use relm4::Component;
use relm4::RelmIterChildrenExt as _;
use relm4::RelmWidgetExt as _;
use relm4::css;
use relm4::prelude::DynamicIndex;
use relm4::prelude::FactoryVecDeque;
use relm4::{ComponentParts, ComponentSender};
use tracing::debug;

pub struct ProfileSelector {
    profiles_info: ProfilesInfo,
    profile_selector: FactoryVecDeque<ProfileRow>,

    new_profile_diag: Option<relm4::Controller<NewProfileDialog>>,
}

#[derive(Debug)]
pub enum ProfileSelectorMsg {
    ClosePopover,
    Profiles(Box<ProfilesInfo>),
    AutoProfileSwitch(bool),
    ShowProfileEditor(DynamicIndex),
    ExportProfile(DynamicIndex),
    RenameProfile(DynamicIndex),
    SelectProfile,
    CreateProfile,
    ImportProfile,
}

#[relm4::component(pub)]
impl Component for ProfileSelector {
    type Init = ();
    type Input = ProfileSelectorMsg;
    type Output = AppMsg;
    type CommandOutput = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            add_css_class: "sidebar-section",

            gtk::Label {
                set_label: "Profile",
                set_halign: gtk::Align::Start,
                set_margin_horizontal: 4,
                add_css_class: css::DIM_LABEL,
                add_css_class: css::CAPTION,
            },

            #[name = "root_menubutton"]
            gtk::MenuButton {
                #[watch]
                set_label: model.selected_profile().unwrap_or("Default"),

                #[wrap(Some)]
                set_popover = &gtk::Popover {
                    add_css_class: "gpu-profile-popover",

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 5,


                    gtk::Label {
                        set_label: &fl!(I18N, "settings-profile"),
                        add_css_class: css::HEADING,
                    },

                    gtk::Box {
                        add_css_class: "profile-picker-container",
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 10,

                        gtk::CheckButton {
                            set_label: Some(&fl!(I18N, "auto-switch-profiles")),
                            #[watch]
                            #[block_signal(toggle_auto_profile_handler)]
                            set_active: model.profiles_info.auto_switch,
                            connect_toggled[sender] => move |button| {
                                sender.input(ProfileSelectorMsg::AutoProfileSwitch(button.is_active()));
                            } @ toggle_auto_profile_handler
                        },

                        gtk::ScrolledWindow {
                            set_policy: (gtk::PolicyType::Never, gtk::PolicyType::Automatic),
                            set_propagate_natural_height: true,

                            #[local_ref]
                            profile_selector -> gtk::ListBox {
                                set_selection_mode: gtk::SelectionMode::Single,
                                add_css_class: css::BOXED_LIST,
                                set_margin_all: 3, // fixes shadow
                            }
                        },

                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 5,

                            gtk::Button {
                                set_icon_name: "list-add",
                                set_expand: true,
                                set_tooltip: &fl!(I18N, "add-profile"),
                                connect_clicked => ProfileSelectorMsg::CreateProfile,
                            },

                            gtk::Button {
                                set_icon_name: "folder-open-symbolic",
                                set_expand: true,
                                set_tooltip: &fl!(I18N, "import-profile"),
                                connect_clicked => ProfileSelectorMsg::ImportProfile,
                            }
                        },
                    },
                    }
                }
            },
        },
    }

    fn init(
        _: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let profile_selector = FactoryVecDeque::<ProfileRow>::builder()
            .launch_default()
            .forward(sender.input_sender(), |msg| msg);
        profile_selector.widget().connect_row_selected(clone!(
            #[strong]
            sender,
            move |_, _| {
                let _ = sender
                    .input_sender()
                    .send(ProfileSelectorMsg::SelectProfile);
            }
        ));

        let model = Self {
            profiles_info: ProfilesInfo::default(),
            profile_selector,
            new_profile_diag: None,
        };

        let profile_selector = model.profile_selector.widget();

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn post_view() {
        // This is horrible, but is needed to access the menubutton's inner label
        let label = root_menubutton
            .first_child()
            .unwrap()
            .first_child()
            .unwrap()
            .first_child()
            .unwrap()
            .downcast::<gtk::Label>()
            .unwrap();

        label.set_xalign(0.0);
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: Self::Input,
        sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match msg {
            ProfileSelectorMsg::ClosePopover => {
                widgets.root_menubutton.popdown();
            }
            ProfileSelectorMsg::Profiles(profiles_info) => {
                self.set_profiles_info(&sender, *profiles_info)
            }
            ProfileSelectorMsg::AutoProfileSwitch(auto_switch) => {
                let msg = AppMsg::SelectProfile {
                    profile: self
                        .selected_profile()
                        .filter(|_| !auto_switch)
                        .map(str::to_owned),
                    auto_switch,
                };
                sender.output(msg).unwrap();
            }
            ProfileSelectorMsg::SelectProfile => {
                let profile = self.selected_profile();

                if self.profiles_info.current_profile.as_deref() != profile {
                    if self.profiles_info.auto_switch {
                        // Revert to the previous profile
                        self.update_selected_profile();
                    } else {
                        sender
                            .output(AppMsg::SelectProfile {
                                profile: profile.map(str::to_owned),
                                auto_switch: false,
                            })
                            .unwrap();
                    }
                }
            }
            ProfileSelectorMsg::ExportProfile(index) => {
                sender.input(ProfileSelectorMsg::ClosePopover);

                let profile = self
                    .profile_selector
                    .get(index.current_index())
                    .expect("No profile with given index");

                let name = match &profile.row {
                    ProfileRowType::Default => None,
                    ProfileRowType::Profile { name, .. } => Some(name.clone()),
                };
                sender.output(AppMsg::ExportProfile(name)).unwrap();
            }
            ProfileSelectorMsg::CreateProfile => {
                sender.input(ProfileSelectorMsg::ClosePopover);

                let diag_controller = NewProfileDialog::builder()
                    .launch((self.custom_profiles(), root.clone().upcast::<gtk::Widget>()))
                    .forward(sender.output_sender(), |(name, base)| {
                        AppMsg::CreateProfile(name, base)
                    });

                self.new_profile_diag = Some(diag_controller);
            }
            ProfileSelectorMsg::RenameProfile(index) => {
                sender.input(ProfileSelectorMsg::ClosePopover);

                let profile = self
                    .profile_selector
                    .get(index.current_index())
                    .expect("No profile with given index");

                let sender = sender.clone();
                if let ProfileRowType::Profile { name, .. } = profile.row.clone() {
                    let stream = ProfileRenameDialog::builder()
                        .launch((name.clone(), root.clone().upcast::<gtk::Widget>()))
                        .into_stream();

                    sender.clone().oneshot_command(async move {
                        if let Some(new_name) = stream.recv_one().await {
                            sender
                                .output(AppMsg::RenameProfile(name, new_name))
                                .unwrap();
                        }
                    });
                }
            }
            ProfileSelectorMsg::ImportProfile => {
                sender.input(ProfileSelectorMsg::ClosePopover);
                sender.output(AppMsg::ImportProfile).unwrap();
            }
            ProfileSelectorMsg::ShowProfileEditor(index) => {
                sender.input(ProfileSelectorMsg::ClosePopover);

                let profile = self
                    .profile_selector
                    .get(index.current_index())
                    .expect("No profile with given index");

                let sender = sender.clone();
                if let ProfileRowType::Profile {
                    name,
                    rule,
                    hooks,
                    auto,
                    ..
                } = &profile.row
                {
                    let params = ProfileEditParams {
                        name: name.clone(),
                        rule: rule.clone().unwrap_or_default(),
                        hooks: hooks.clone(),
                        auto_switch: *auto,
                        parent: root.clone().upcast::<gtk::Widget>(),
                    };
                    let rule_window = ProfileRuleWindow::builder().launch(params).into_stream();

                    sender.clone().oneshot_command(async move {
                        if let Some((name, rule, hooks)) = rule_window.recv_one().await {
                            sender
                                .output(AppMsg::SetProfileRule {
                                    name,
                                    rule: Some(rule),
                                    hooks,
                                })
                                .unwrap();
                        }
                    });
                }
            }
        }

        self.update_view(widgets, sender);
    }
}

impl ProfileSelector {
    pub fn auto_switch_profiles(&self) -> bool {
        self.profiles_info.auto_switch
    }

    fn set_profiles_info(&mut self, sender: &ComponentSender<Self>, profiles_info: ProfilesInfo) {
        if self.profiles_info == profiles_info && !self.profile_selector.is_empty() {
            return;
        }
        debug!("setting new profiles info: {profiles_info:?}");

        sender.output(AppMsg::ReloadData { full: false }).unwrap();

        let mut profiles = self.profile_selector.guard();
        profiles.clear();

        let last = profiles_info.profiles.len().saturating_sub(1);
        for (i, (name, rule)) in profiles_info.profiles.iter().enumerate() {
            let hooks = profiles_info
                .profile_hooks
                .get(name)
                .cloned()
                .unwrap_or_default();

            let profile = ProfileRowType::Profile {
                name: name.to_string(),
                first: i == 0,
                last: i == last,
                auto: profiles_info.auto_switch,
                rule: rule.clone(),
                hooks,
            };
            profiles.push_back(profile);
        }
        profiles.push_back(ProfileRowType::Default);
        drop(profiles);

        self.profiles_info = profiles_info;

        self.update_selected_profile();

        if self.auto_switch_profiles() {
            let profiles_listbox = self.profile_selector.widget();
            for row in profiles_listbox.iter_children() {
                row.remove_css_class("activatable");
            }
        }
    }

    fn update_selected_profile(&self) {
        let selected_profile_index = self.profiles_info.current_profile.as_ref().map(|profile| {
            self.profiles_info
                .profiles
                .iter()
                .position(|(value, _)| value == profile)
                .expect("Active profile is not in the list")
        });

        let new_selected_index =
            selected_profile_index.unwrap_or_else(|| self.profile_selector.len() - 1);

        let new_selected_row = self
            .profile_selector
            .widget()
            .row_at_index(new_selected_index as i32)
            .unwrap();

        self.profile_selector
            .widget()
            .select_row(Some(&new_selected_row));
    }

    fn custom_profiles(&self) -> Vec<String> {
        let mut profiles = Vec::with_capacity(self.profile_selector.len());
        for i in 0..self.profile_selector.len() {
            let item = self.profile_selector.get(i).unwrap();
            if let ProfileRowType::Profile { name, .. } = &item.row {
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
            .and_then(|item| match &item.row {
                ProfileRowType::Default => None,
                ProfileRowType::Profile { name, .. } => Some(name.as_str()),
            })
    }
}
