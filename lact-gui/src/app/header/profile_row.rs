use super::HeaderMsg;
use crate::app::{msg::AppMsg, APP_BROKER};
use crate::I18N;
use gtk::{pango, prelude::*};
use i18n_embed_fl::fl;
use lact_schema::{config::ProfileHooks, ProfileRule};
use relm4::{
    css,
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
        hooks: ProfileHooks,
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
            set_spacing: 5,
            set_margin_horizontal: 5,

            #[name = "name_label"]
            gtk::Label {
                set_label: &match &self.row {
                    ProfileRowType::Default => fl!(I18N, "default-profile"),
                    ProfileRowType::Profile { name, .. } => name.clone(),
                },
                set_halign: gtk::Align::Start,
                set_hexpand: true,
                set_xalign: 0.0,
                set_ellipsize: pango::EllipsizeMode::End,
                set_width_request: 200,
            },

            gtk::MenuButton {
                set_icon_name: "open-menu-symbolic",
                #[wrap(Some)]
                set_popover = &gtk::Popover {
                    set_margin_all: 5,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 5,

                        gtk::Button {
                            set_label: &fl!(I18N, "rename-profile"),
                            set_sensitive: matches!(self.row, ProfileRowType::Profile { .. }),
                            connect_clicked[sender, index] => move |_| {
                                sender.output(HeaderMsg::RenameProfile(index.clone())).unwrap();
                            },
                            add_css_class: css::FLAT,
                        },

                        gtk::Button {
                            set_label: &fl!(I18N, "delete-profile"),
                            set_sensitive: matches!(self.row, ProfileRowType::Profile { .. }),
                            connect_clicked[profile = self.row.clone()] => move |_| {
                                if let ProfileRowType::Profile { name, .. } = profile.clone() {
                                    APP_BROKER.send(AppMsg::DeleteProfile(name));
                                }
                            },
                            add_css_class: css::FLAT,
                        },

                        gtk::Button {
                            set_label: &fl!(I18N, "edit-rules"),
                            connect_clicked[sender, index] => move |_| {
                                sender.output(HeaderMsg::ShowProfileEditor(index.clone())).unwrap();
                            },
                            add_css_class: css::FLAT,
                        },

                        gtk::Button {
                            set_label: &fl!(I18N, "export-to-file"),
                            connect_clicked[sender, index] => move |_| {
                                sender.output(HeaderMsg::ExportProfile(index.clone())).unwrap();
                            },
                            add_css_class: css::FLAT,
                        },
                    },
                }
            },

            gtk::Button {
                set_icon_name: "go-up",
                set_tooltip: &fl!(I18N, "move-up"),
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
                set_tooltip: &fl!(I18N, "move-down"),
                set_sensitive: match &self.row {
                    ProfileRowType::Profile { last, .. } => !*last,
                    _ => false,

                },
                connect_clicked[index, profile = self.row.clone()] => move |_| {
                    APP_BROKER.send(move_profile_msg(&profile, &index, 1));
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
