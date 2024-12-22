mod rule_window;

use gtk::{pango, prelude::*};
use lact_schema::ProfileRule;
use relm4::{
    factory::{DynamicIndex, FactoryComponent},
    Component, ComponentController, FactorySender, RelmWidgetExt,
};
use rule_window::RuleWindow;

use crate::app::{msg::AppMsg, APP_BROKER};

use super::HeaderMsg;

#[derive(Clone, Debug)]
pub enum ProfileRow {
    Default,
    Profile {
        name: String,
        first: bool,
        last: bool,
        auto: bool,
        rule: Option<ProfileRule>,
    },
}

impl ProfileRow {
    pub fn name(&self) -> Option<String> {
        match self {
            ProfileRow::Default => None,
            ProfileRow::Profile { name, .. } => Some(name.clone()),
        }
    }
}

#[derive(Debug)]
pub enum ProfileRowMsg {
    EditRule,
}

#[relm4::factory(pub)]
impl FactoryComponent for ProfileRow {
    type Init = Self;
    type Input = ProfileRowMsg;
    type Output = HeaderMsg;
    type CommandOutput = ();
    type ParentWidget = gtk::ListBox;

    view! {
        gtk::Box {
            #[name = "name_label"]
            gtk::Label {
                set_label: match self {
                    ProfileRow::Default => "Default",
                    ProfileRow::Profile { name, .. } => name,
                },
                set_margin_all: 5,
                set_halign: gtk::Align::Start,
                set_hexpand: true,
                set_xalign: 0.0,
                set_ellipsize: pango::EllipsizeMode::End,
                set_width_request: 200,
            },

            gtk::Button {
                set_icon_name: "preferences-other-symbolic",
                set_tooltip: "Edit Profile Rules",
                set_sensitive: matches!(self, ProfileRow::Profile { auto: true, .. }),
                connect_clicked => ProfileRowMsg::EditRule,
            },

            gtk::Button {
                set_icon_name: "go-up",
                set_tooltip: "Move Up",
                set_sensitive: match self {
                    ProfileRow::Profile { first, .. } => !*first,
                    _ => false,

                },
                connect_clicked[index, profile = self.clone()] => move |_| {
                    APP_BROKER.send(move_profile_msg(&profile, &index, -1));
                },
            },

            gtk::Button {
                set_icon_name: "go-down",
                set_tooltip: "Move Down",
                set_sensitive: match self {
                    ProfileRow::Profile { last, .. } => !*last,
                    _ => false,

                },
                connect_clicked[index, profile = self.clone()] => move |_| {
                    APP_BROKER.send(move_profile_msg(&profile, &index, 1));
                },
            },

            gtk::Button {
                set_icon_name: "list-remove",
                set_sensitive: matches!(self, ProfileRow::Profile { .. }),
                set_tooltip: "Delete Profile",
                connect_clicked[profile = self.clone()] => move |_| {
                    if let ProfileRow::Profile { name, .. } = profile.clone() {
                        APP_BROKER.send(AppMsg::DeleteProfile(name));
                    }
                },
            },
        }
    }

    fn init_model(model: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        model
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: Self::Input,
        sender: FactorySender<Self>,
    ) {
        match msg {
            ProfileRowMsg::EditRule => {
                if let Self::Profile { rule, name, .. } = self {
                    sender.output(HeaderMsg::ClosePopover).unwrap();

                    let mut rule_window = RuleWindow::builder()
                        .transient_for(&widgets.name_label)
                        .launch((rule.clone(), name.clone()));
                    rule_window.detach_runtime();
                }
            }
        }
    }
}

fn move_profile_msg(profile: &ProfileRow, index: &DynamicIndex, offset: i64) -> AppMsg {
    let name = profile.name().expect("Default profile cannot be moved");
    let new_index = (index.current_index() as i64).saturating_add(offset);
    AppMsg::MoveProfile(name, new_index as usize)
}
