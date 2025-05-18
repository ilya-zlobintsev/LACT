pub mod profile_row;

use crate::app::{msg::AppMsg, APP_BROKER};
use gtk::prelude::{
    BoxExt, ButtonExt, CheckButtonExt, DialogExt, DialogExtManual, GtkWindowExt, OrientableExt,
    WidgetExt,
};
use lact_schema::ProfileRule;
use profile_row::ProfileRuleRow;
use relm4::{
    prelude::{DynamicIndex, FactoryVecDeque},
    tokio::time::sleep,
    ComponentParts, ComponentSender, RelmWidgetExt,
};
use std::time::Duration;

const EVALUATE_INTERVAL_MS: u64 = 250;

pub struct ProfileRuleWindow {
    profile_name: String,
    sub_rules_list_view: FactoryVecDeque<ProfileRuleRow>,
    currently_matches: bool,
}

#[derive(Debug)]
pub enum ProfileRuleWindowMsg {
    Evaluate,
    EvaluationResult(bool),
    AddSubrule,
    RemoveSubrule(DynamicIndex),
    Save,
}

#[relm4::component(pub)]
impl relm4::Component for ProfileRuleWindow {
    type Init = (String, ProfileRule);
    type Input = ProfileRuleWindowMsg;
    type Output = (String, ProfileRule);
    type CommandOutput = ();

    view! {
        gtk::Dialog {
            set_default_size: (600, 300),
            set_title: Some("Profile activation rules"),
            connect_response[root, sender] => move |_, response| {
                match response {
                    gtk::ResponseType::Accept => {
                        sender.input(ProfileRuleWindowMsg::Save);
                        root.close();
                    }
                    gtk::ResponseType::Cancel => root.close(),
                    _ => (),
                }
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 5,

                gtk::Label {
                    #[watch]
                    set_markup: &format!("<span font_desc='11'><b>Activate profile '{}' when:</b></span>", model.profile_name),
                    set_halign: gtk::Align::Start,
                    set_margin_all: 10,
                },

                gtk::Separator {},

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_expand: true,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_margin_all: 10,
                        set_spacing: 10,

                        #[name = "multi_or_checkbutton"]
                        gtk::CheckButton {
                            set_label: Some("Any of the following rules are matched:"),
                            set_active: !matches!(rule, ProfileRule::And(_)),
                            connect_toggled => ProfileRuleWindowMsg::Evaluate,
                        },

                        #[name = "multi_and_checkbutton"]
                        gtk::CheckButton {
                            set_label: Some("All of the following rules are matched:"),
                            set_group: Some(&multi_or_checkbutton),
                            set_active: matches!(rule, ProfileRule::And(_)),
                            connect_toggled => ProfileRuleWindowMsg::Evaluate,
                        },

                        gtk::Separator {},

                        #[local_ref]
                        sub_rules_listview -> gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 5,
                        },

                        gtk::Button {
                            set_icon_name: "list-add-symbolic",
                            set_hexpand: true,
                            set_halign: gtk::Align::End,
                            connect_clicked => ProfileRuleWindowMsg::AddSubrule,
                        },

                        gtk::Separator {},

                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 5,

                            gtk::Label {
                                #[watch]
                                set_markup: &format!(
                                    "Selected activation settings are currently <b>{}</b>",
                                    if model.currently_matches { "matched" } else { "not matched" }
                                ),
                            },

                            gtk::Image {
                                #[watch]
                                set_icon_name: match model.currently_matches {
                                    true => Some("object-select-symbolic"),
                                    false => Some("list-remove-symbolic"),
                                }
                            },
                        }
                    },
                },

                gtk::Separator {},
            },

            add_buttons: &[("Cancel", gtk::ResponseType::Cancel), ("Save", gtk::ResponseType::Accept)],
        }
    }

    fn init(
        (profile_name, rule): Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let task_sender = sender.clone();
        sender.command(move |_, shutdown| {
            shutdown
                .register(async move {
                    loop {
                        sleep(Duration::from_millis(EVALUATE_INTERVAL_MS)).await;
                        task_sender.input(ProfileRuleWindowMsg::Evaluate);
                    }
                })
                .drop_on_shutdown()
        });

        let mut sub_rules_list_view = FactoryVecDeque::builder()
            .launch_default()
            .forward(sender.input_sender(), |msg| msg);

        match &rule {
            ProfileRule::And(subrules) | ProfileRule::Or(subrules) => {
                for rule in subrules.iter().cloned() {
                    sub_rules_list_view.guard().push_back(rule);
                }
            }
            rule => {
                sub_rules_list_view.guard().push_back(rule.clone());
            }
        };

        let model = Self {
            profile_name,
            sub_rules_list_view,
            currently_matches: false,
        };

        let sub_rules_listview = model.sub_rules_list_view.widget();
        let widgets = view_output!();

        root.present();

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
            ProfileRuleWindowMsg::Evaluate => {
                if root.is_visible() {
                    let rule = self.get_rule(widgets);
                    APP_BROKER.send(AppMsg::EvaluateProfile(rule, sender.input_sender().clone()));
                }
            }
            ProfileRuleWindowMsg::AddSubrule => {
                self.sub_rules_list_view
                    .guard()
                    .push_back(ProfileRule::default());
            }
            ProfileRuleWindowMsg::RemoveSubrule(index) => {
                self.sub_rules_list_view
                    .guard()
                    .remove(index.current_index());
            }
            ProfileRuleWindowMsg::EvaluationResult(matches) => {
                self.currently_matches = matches;
            }
            ProfileRuleWindowMsg::Save => {
                sender
                    .output((self.profile_name.clone(), self.get_rule(widgets)))
                    .unwrap();
            }
        }

        self.update_view(widgets, sender);
    }
}

impl ProfileRuleWindow {
    fn get_rule(&self, widgets: &ProfileRuleWindowWidgets) -> ProfileRule {
        let rules = self
            .sub_rules_list_view
            .iter()
            .map(|row| row.get_configured_rule())
            .collect::<Vec<_>>();

        if rules.len() == 1 {
            rules.into_iter().next().unwrap()
        } else if widgets.multi_or_checkbutton.is_active() {
            ProfileRule::Or(rules)
        } else {
            ProfileRule::And(rules)
        }
    }
}
