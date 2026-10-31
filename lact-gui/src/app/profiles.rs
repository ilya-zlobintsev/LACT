mod new_profile_dialog;
mod profile_rename_dialog;
mod profile_row;
pub mod profile_rule_window;

use crate::I18N;
use crate::app::components::checkbox_button::CheckboxButton;
use crate::app::msg::AppMsg;
use crate::app::profiles::new_profile_dialog::NewProfileDialog;
use crate::app::profiles::profile_rename_dialog::ProfileRenameDialog;
use crate::app::profiles::profile_row::ProfileRow;
use crate::app::profiles::profile_row::ProfileRowMsg;
use crate::app::profiles::profile_row::ProfileRowType;
use crate::app::profiles::profile_rule_window::ProfileEditParams;
use crate::app::profiles::profile_rule_window::ProfileRuleWindow;
use crate::app::utils::ext::RelmLaunchable as _;
use gtk::prelude::*;
use i18n_embed_fl::fl;
use lact_schema::ProfilesInfo;
use relm4::Component;
use relm4::RelmWidgetExt as _;
use relm4::css;
use relm4::prelude::DynamicIndex;
use relm4::prelude::FactoryVecDeque;
use relm4::{ComponentParts, ComponentSender};
use std::sync::Arc;
use tracing::debug;

pub struct ProfileSelector {
    profiles_info: Arc<ProfilesInfo>,
    profile_selector: FactoryVecDeque<ProfileRow>,

    new_profile_diag: Option<relm4::Controller<NewProfileDialog>>,
}

#[derive(Debug)]
pub enum ProfileSelectorMsg {
    ClosePopover,
    Profiles(Arc<ProfilesInfo>),
    AutoProfileSwitch(bool),
    ShowProfileEditor(DynamicIndex),
    ExportProfile(DynamicIndex),
    RenameProfile(DynamicIndex),
    SelectProfile(usize),
    MoveProfile {
        from: DynamicIndex,
        to: DynamicIndex,
    },
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
                    set_width_request: 230,

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

                        #[template]
                        CheckboxButton {
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
                                set_selection_mode: gtk::SelectionMode::None,
                                add_css_class: css::BOXED_LIST,
                                set_margin_all: 3, // fixes shadow
                                connect_row_activated[sender] => move |_, row| {
                                    sender.input(ProfileSelectorMsg::SelectProfile(row.index() as usize));
                                },
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
        let model = Self {
            profiles_info: Arc::new(ProfilesInfo::default()),
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
                self.set_profiles_info(profiles_info);
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
            ProfileSelectorMsg::SelectProfile(index) => {
                let profile = self
                    .profile_selector
                    .get(index)
                    .expect("No profile with given index");

                if !profile.selected {
                    let profile = profile.row.name();

                    // updates rows selected status optimistically before daemon replies
                    for i in 0..self.profile_selector.len() {
                        self.profile_selector
                            .send(i, ProfileRowMsg::Selected(i == index));
                    }
                    sender
                        .output(AppMsg::SelectProfile {
                            profile,
                            auto_switch: false,
                        })
                        .unwrap();
                }
            }
            ProfileSelectorMsg::MoveProfile { from, to } => {
                let (from, to) = (from.current_index(), to.current_index());

                if from != to {
                    let name = self
                        .profile_selector
                        .get(from)
                        .expect("No profile with given index")
                        .row
                        .name()
                        .expect("Default profile cannot be moved");

                    // moves the row optimistically before daemon replies
                    self.profile_selector.guard().move_to(from, to);
                    sender.output(AppMsg::MoveProfile(name, to)).unwrap();
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
                    name, rule, hooks, ..
                } = &profile.row
                {
                    let params = ProfileEditParams {
                        name: name.clone(),
                        rule: rule.clone().unwrap_or_default(),
                        hooks: hooks.clone(),
                        auto_switch: self.profiles_info.auto_switch,
                        parent: root.clone().upcast::<gtk::Widget>(),
                    };
                    let rule_window = ProfileRuleWindow::launch(params).into_stream();

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

    pub fn profiles_info_changed(&self, profiles_info: &ProfilesInfo) -> bool {
        self.profiles_info.as_ref() != profiles_info
    }

    fn set_profiles_info(&mut self, profiles_info: Arc<ProfilesInfo>) {
        if self.profiles_info.as_ref() == profiles_info.as_ref()
            && !self.profile_selector.is_empty()
        {
            return;
        }
        debug!("setting new profiles info: {profiles_info:?}");

        let mut profiles = self.profile_selector.guard();
        profiles.clear();

        for (name, rule) in &profiles_info.profiles {
            let hooks = profiles_info
                .profile_hooks
                .get(name)
                .cloned()
                .unwrap_or_default();

            profiles.push_back(ProfileRow {
                row: ProfileRowType::Profile {
                    name: name.to_string(),
                    rule: rule.clone(),
                    hooks,
                },
                selected: profiles_info.current_profile.as_ref() == Some(name),
                activatable: !profiles_info.auto_switch,
            });
        }
        profiles.push_back(ProfileRow {
            row: ProfileRowType::Default,
            selected: profiles_info.current_profile.is_none(),
            activatable: !profiles_info.auto_switch,
        });
        drop(profiles);

        self.profiles_info = profiles_info;
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
        self.profiles_info.current_profile.as_deref()
    }
}
