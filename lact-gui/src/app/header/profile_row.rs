use super::HeaderMsg;
use crate::app::{msg::AppMsg, APP_BROKER};
use gtk::{pango, prelude::*};
use lact_schema::ProfileRule;
use relm4::{
    factory::{DynamicIndex, FactoryComponent},
    FactorySender, RelmWidgetExt,
};

pub struct ProfileRow {
    pub(super) row: ProfileRowType,
}

#[derive(Debug, Clone)]
pub enum ProfileRowType {
    Default,
    Profile {
        name: String,
        first: bool,
        last: bool,
        auto: bool,
        rule: Option<ProfileRule>,
    },
}

impl ProfileRowType {
    pub fn name(&self) -> Option<String> {
        match self {
            Self::Default => None,
            Self::Profile { name, .. } => Some(name.clone()),
        }
    }
}

#[relm4::factory(pub)]
impl FactoryComponent for ProfileRow {
    type Init = ProfileRowType;
    type Input = ();
    type Output = HeaderMsg;
    type CommandOutput = ();
    type ParentWidget = gtk::ListBox;

    view! {
        gtk::Box {
            #[name = "name_label"]
            gtk::Label {
                set_label: match &self.row {
                    ProfileRowType::Default => "Default",
                    ProfileRowType::Profile { name, .. } => name,
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
                set_sensitive: matches!(self.row, ProfileRowType::Profile { auto: true, .. }),
                connect_clicked[sender, index] => move |_| {
                    sender.output(HeaderMsg::ShowProfileEditor(index.clone())).unwrap();
                }
            },

            gtk::Button {
                set_icon_name: "go-up",
                set_tooltip: "Move Up",
                set_sensitive: match &self.row {
                    ProfileRowType::Profile { first, .. } => !*first,
                    _ => false,

                },
                connect_clicked[index, profile = self.row.clone()] => move |_| {
                    APP_BROKER.send(move_profile_msg(&profile, &index, -1));
                },
            },

            gtk::Button {
                set_icon_name: "go-down",
                set_tooltip: "Move Down",
                set_sensitive: match &self.row {
                    ProfileRowType::Profile { last, .. } => !*last,
                    _ => false,

                },
                connect_clicked[index, profile = self.row.clone()] => move |_| {
                    APP_BROKER.send(move_profile_msg(&profile, &index, 1));
                },
            },

            gtk::Button {
                set_icon_name: "list-remove",
                set_sensitive: matches!(self.row, ProfileRowType::Profile { .. }),
                set_tooltip: "Delete Profile",
                connect_clicked[profile = self.row.clone()] => move |_| {
                    if let ProfileRowType::Profile { name, .. } = profile.clone() {
                        APP_BROKER.send(AppMsg::DeleteProfile(name));
                    }
                },
            },
        }
    }

    fn init_model(row: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self { row }
    }
}

fn move_profile_msg(profile: &ProfileRowType, index: &DynamicIndex, offset: i64) -> AppMsg {
    let name = profile.name().expect("Default profile cannot be moved");
    let new_index = (index.current_index() as i64).saturating_add(offset);
    AppMsg::MoveProfile(name, new_index as usize)
}
