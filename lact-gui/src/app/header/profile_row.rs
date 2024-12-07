use gtk::prelude::*;
use relm4::{
    factory::{DynamicIndex, FactoryComponent},
    FactorySender, RelmWidgetExt,
};

#[derive(Clone, Debug)]
pub enum ProfileRow {
    Default,
    Profile {
        name: String,
        first: bool,
        last: bool,
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
pub enum ProfileRowOutput {
    MoveUp(ProfileRow, DynamicIndex),
    MoveDown(ProfileRow, DynamicIndex),
    Delete(String),
}

#[relm4::factory(pub)]
impl FactoryComponent for ProfileRow {
    type Init = Self;
    type Input = ();
    type Output = ProfileRowOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::ListBox;

    view! {
        gtk::Box {
            gtk::Label {
                set_label: match self {
                    ProfileRow::Default => "Default",
                    ProfileRow::Profile { name, .. } => name,
                },
                set_margin_all: 5,
                set_hexpand: true,
                set_xalign: 0.1,
            },

            gtk::Button {
                set_icon_name: "go-up",
                set_sensitive: match self {
                    ProfileRow::Profile { first, .. } => !*first,
                    _ => false,

                },
                connect_clicked[sender, index, profile = self.clone()] => move |_| {
                    sender.output(ProfileRowOutput::MoveUp(profile.clone(), index.clone())).unwrap();
                },
            },

            gtk::Button {
                set_icon_name: "go-down",
                set_sensitive: match self {
                    ProfileRow::Profile { last, .. } => !*last,
                    _ => false,

                },
                connect_clicked[sender, index, profile = self.clone()] => move |_| {
                    sender.output(ProfileRowOutput::MoveDown(profile.clone(), index.clone())).unwrap();
                },
            },

            gtk::Button {
                set_icon_name: "list-remove",
                set_sensitive: matches!(self, ProfileRow::Profile { .. }),
                connect_clicked[sender, profile = self.clone()] => move |_| {
                    if let ProfileRow::Profile { name, .. } = profile.clone() {
                        sender.output(ProfileRowOutput::Delete(name)).unwrap();
                    }
                },
            },
        }
    }

    fn init_model(model: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        model
    }
}
