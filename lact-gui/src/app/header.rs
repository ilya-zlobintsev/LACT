mod new_profile_dialog;
mod profile_rename_dialog;
mod profile_row;
pub mod profile_rule_window;

use crate::{
    app::{header::profile_rule_window::ProfileEditParams, ShowProcessMonitor, APP_BROKER},
    CONFIG, I18N,
};

use super::{AppMsg, DebugSnapshot, DisableOverdrive, DumpVBios, ResetConfig, ShowGraphsWindow};
use glib::clone;
use gtk::prelude::*;
use gtk::*;
use i18n_embed_fl::fl;
use lact_client::schema::DeviceListEntry;
use lact_schema::ProfilesInfo;
use new_profile_dialog::NewProfileDialog;
use profile_rename_dialog::ProfileRenameDialog;
use profile_row::{ProfileRow, ProfileRowType};
use profile_rule_window::ProfileRuleWindow;
use relm4::{
    factory::FactoryVecDeque,
    prelude::DynamicIndex,
    typed_view::list::{RelmListItem, TypedListView},
    Component, ComponentController, ComponentParts, ComponentSender, RelmIterChildrenExt,
    RelmWidgetExt,
};
use tracing::debug;

pub struct Header {
    profiles_info: ProfilesInfo,
    gpu_selector: TypedListView<GpuListItem, gtk::SingleSelection>,
    profile_selector: FactoryVecDeque<ProfileRow>,
    selector_label: String,
}

#[derive(Debug)]
pub enum HeaderMsg {
    Profiles(std::boxed::Box<ProfilesInfo>),
    AutoProfileSwitch(bool),
    ShowProfileEditor(DynamicIndex),
    ExportProfile(DynamicIndex),
    RenameProfile(DynamicIndex),
    SelectProfile,
    SelectGpu,
    CreateProfile,
    ImportProfile,
    ClosePopover,
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
            set_title_widget: stack_switcher = &StackSwitcher {},

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
                            set_label: Some(&fl!(I18N, "settings-profile")),
                            set_label_align: 0.05,
                            set_margin_all: 5,

                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_spacing: 5,

                                gtk::CheckButton {
                                    set_label: Some(&fl!(I18N, "auto-switch-profiles")),
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
                                        set_tooltip: &fl!(I18N, "add-profile"),
                                        connect_clicked => HeaderMsg::CreateProfile,
                                    },

                                    gtk::Button {
                                        set_icon_name: "document-import-symbolic",
                                        set_tooltip: &fl!(I18N, "import-profile"),
                                        set_expand: true,
                                        connect_clicked => HeaderMsg::ImportProfile,
                                    }
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
                "Show process monitor" => ShowProcessMonitor,
            },
            section! {
                "Generate debug snapshot" => DebugSnapshot,
                "Dump VBIOS" => DumpVBios,
            } ,
            section! {
                "Disable AMD overclocking support" => DisableOverdrive,
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

        if let Some(selected_gpu_id) = &CONFIG.read().selected_gpu {
            for idx in 0..gpu_selector.len() {
                let gpu_item = gpu_selector.get(idx).unwrap();
                if gpu_item.borrow().0.id == *selected_gpu_id {
                    debug!("selecting gpu id {selected_gpu_id}");
                    gpu_selector.selection_model.set_selected(idx);
                    break;
                }
            }
        }

        let profile_selector = FactoryVecDeque::<ProfileRow>::builder()
            .launch_default()
            .forward(sender.input_sender(), |msg| msg);
        profile_selector.widget().connect_row_selected(clone!(
            #[strong]
            sender,
            move |_, _| {
                let _ = sender.input_sender().send(HeaderMsg::SelectProfile);
            }
        ));

        let model = Self {
            gpu_selector,
            profile_selector,
            selector_label: String::new(),
            profiles_info: ProfilesInfo::default(),
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
        root: &Self::Root,
    ) {
        match msg {
            HeaderMsg::ClosePopover => {
                widgets.menu_button.popdown();
            }
            HeaderMsg::Profiles(profiles_info) => self.set_profiles_info(*profiles_info),
            HeaderMsg::SelectGpu => {
                let gpu_id = self.selected_gpu_id();
                CONFIG.write().edit(|config| {
                    config.selected_gpu = gpu_id;
                });

                sender.output(AppMsg::ReloadData { full: true }).unwrap()
            }
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
            HeaderMsg::ExportProfile(index) => {
                sender.input(HeaderMsg::ClosePopover);

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
            HeaderMsg::CreateProfile => {
                sender.input(HeaderMsg::ClosePopover);

                let mut diag_controller = NewProfileDialog::builder()
                    .launch(self.custom_profiles())
                    .forward(sender.output_sender(), |(name, base)| {
                        AppMsg::CreateProfile(name, base)
                    });
                diag_controller.detach_runtime();
            }
            HeaderMsg::RenameProfile(index) => {
                sender.input(HeaderMsg::ClosePopover);

                let profile = self
                    .profile_selector
                    .get(index.current_index())
                    .expect("No profile with given index");

                let sender = sender.clone();
                if let ProfileRowType::Profile { name, .. } = profile.row.clone() {
                    let stream = ProfileRenameDialog::builder()
                        .launch((
                            name.clone(),
                            root.toplevel_window().expect("Widget not in a window"),
                        ))
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
            HeaderMsg::ImportProfile => {
                sender.input(HeaderMsg::ClosePopover);
                sender.output(AppMsg::ImportProfile).unwrap();
            }
            HeaderMsg::ShowProfileEditor(index) => {
                sender.input(HeaderMsg::ClosePopover);

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
                        root_window: root.toplevel_window().expect("Widget not in a window"),
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
        self.update_label();

        self.update_view(widgets, sender);
    }
}

impl Header {
    fn set_profiles_info(&mut self, profiles_info: ProfilesInfo) {
        if self.profiles_info == profiles_info && !self.profile_selector.is_empty() {
            return;
        }
        debug!("setting new profiles info: {profiles_info:?}");

        APP_BROKER.send(AppMsg::ReloadData { full: false });

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

    fn update_label(&mut self) {
        let gpu_index = self.gpu_selector.selection_model.selected();
        let profile = self.selected_profile().unwrap_or("Default");

        self.selector_label = format!("GPU {gpu_index} | {profile}");
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
