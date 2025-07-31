pub mod profile_row;

use crate::app::{msg::AppMsg, APP_BROKER};
use crate::I18N;
use gtk::{
    pango,
    prelude::{
        BoxExt, ButtonExt, CheckButtonExt, DialogExt, DialogExtManual, EntryBufferExtManual,
        EntryExt, GtkWindowExt, OrientableExt, WidgetExt,
    },
};
use i18n_embed_fl::fl;
use lact_schema::{config::ProfileHooks, ProfileRule};
use profile_row::ProfileRuleRow;
use relm4::{
    binding::BoolBinding,
    prelude::{DynamicIndex, FactoryVecDeque},
    tokio::time::sleep,
    ComponentParts, ComponentSender, RelmObjectExt, RelmWidgetExt,
};
use std::time::Duration;

const EVALUATE_INTERVAL_MS: u64 = 250;

pub struct ProfileRuleWindow {
    profile_name: String,
    sub_rules_list_view: FactoryVecDeque<ProfileRuleRow>,
    currently_matches: bool,
    auto_switch: bool,

    activated_hook_enabled: BoolBinding,
    activated_hook: gtk::EntryBuffer,

    deactivated_hook_enabled: BoolBinding,
    deactivated_hook: gtk::EntryBuffer,
}

pub struct ProfileEditParams {
    pub name: String,
    pub rule: ProfileRule,
    pub hooks: ProfileHooks,
    pub auto_switch: bool,
    pub root_window: gtk::Window,
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
    type Init = ProfileEditParams;
    type Input = ProfileRuleWindowMsg;
    type Output = (String, ProfileRule, ProfileHooks);
    type CommandOutput = ();

    view! {
        gtk::Dialog {
            set_default_size: (600, 300),
            set_title: Some(&fl!(I18N, "profile-rules")),
            set_transient_for: Some(&root_window),
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

                append = &gtk::StackSwitcher {
                    set_stack: Some(&stack),
                },

                #[name = "stack"]
                append = &gtk::Stack {
                    add_titled[None, &fl!(I18N, "profile-activation")] = &gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_margin_all: 5,

                        gtk::Label {
                            #[watch]
                            set_markup: &fl!(I18N, "profile-activation-desc", name = model.profile_name.as_str()),
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
                                    set_label: Some(&fl!(I18N, "any-rules-matched")),
                                    set_active: !matches!(rule, ProfileRule::And(_)),
                                    connect_toggled => ProfileRuleWindowMsg::Evaluate,
                                },

                                #[name = "multi_and_checkbutton"]
                                gtk::CheckButton {
                                    set_label: Some(&fl!(I18N, "all-rules-matched")),
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
                                        set_markup: &if model.auto_switch {
                                            format!(
                                                "Selected activation settings are currently <b>{}</b>",
                                                if model.currently_matches { "matched" } else { "not matched" }
                                            )
                                        } else {
                                            "<b>Automatic profile switching is currently disabled</b>".to_owned()
                                        },
                                    },

                                    gtk::Image {
                                        #[watch]
                                        set_icon_name: match model.currently_matches {
                                            true => Some("object-select-symbolic"),
                                            false => Some("list-remove-symbolic"),
                                        },
                                    },
                                }
                            },
                        },

                        gtk::Separator {},
                    },

                    add_titled[None, "Hooks"] = &gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_margin_all: 5,

                        gtk::Label {
                            #[watch]
                            set_markup: &format!("<span font_desc='11'><b>Run a command when the profile '{}' is:</b></span>", model.profile_name),
                            set_halign: gtk::Align::Start,
                            set_margin_all: 10,
                        },

                        gtk::Separator {},

                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 5,
                            set_margin_vertical: 5,
                            set_margin_horizontal: 10,


                            gtk::CheckButton {
                                set_label: Some("Activated:"),
                                add_binding: (&model.activated_hook_enabled, "active"),
                                set_size_group: &hook_command_size_group,
                            },

                            gtk::Entry {
                                add_binding: (&model.activated_hook_enabled, "sensitive"),
                                set_buffer: &model.activated_hook,
                                set_hexpand: true,
                            },
                        },

                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 5,
                            set_margin_vertical: 5,
                            set_margin_horizontal: 10,

                            gtk::CheckButton {
                                set_label: Some("Deactivated:"),
                                add_binding: (&model.deactivated_hook_enabled, "active"),
                                set_size_group: &hook_command_size_group,
                            },

                            gtk::Entry {
                                add_binding: (&model.deactivated_hook_enabled, "sensitive"),
                                set_buffer: &model.deactivated_hook,
                                set_hexpand: true,
                            },
                        },

                        gtk::Separator {},

                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 5,
                            set_margin_vertical: 5,
                            set_margin_horizontal: 10,

                            gtk::Image {
                                set_icon_name: Some("dialog-warning-symbolic"),
                            },

                            gtk::Label {
                                set_label: "Note: these commands are executed as root by the LACT daemon, and do not have access to the desktop environment. As such, they cannot be used directly to launch graphical applications.",
                                set_wrap: true,
                                set_wrap_mode: pango::WrapMode::Word,
                                add_css_class: "caption-heading",
                                set_hexpand: true,
                            },
                        }
                    },
                }
            },

            add_buttons: &[("Cancel", gtk::ResponseType::Cancel), ("Save", gtk::ResponseType::Accept)],
        }
    }

    fn init(
        params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let task_sender = sender.clone();
        let ProfileEditParams {
            name,
            rule,
            hooks,
            auto_switch,
            root_window,
        } = params;

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
            profile_name: name,
            sub_rules_list_view,
            currently_matches: false,
            auto_switch,
            activated_hook_enabled: BoolBinding::new(hooks.activated.is_some()),
            activated_hook: gtk::EntryBuffer::new(hooks.activated),
            deactivated_hook_enabled: BoolBinding::new(hooks.deactivated.is_some()),
            deactivated_hook: gtk::EntryBuffer::new(hooks.deactivated),
        };

        let sub_rules_listview = model.sub_rules_list_view.widget();
        let hook_command_size_group = gtk::SizeGroup::new(gtk::SizeGroupMode::Horizontal);
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
                if root.is_visible() && self.auto_switch {
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
                    .output((
                        self.profile_name.clone(),
                        self.get_rule(widgets),
                        self.get_hooks(),
                    ))
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

    fn get_hooks(&self) -> ProfileHooks {
        ProfileHooks {
            activated: if self.activated_hook_enabled.value() {
                Some(self.activated_hook.text().to_string())
            } else {
                None
            },
            deactivated: if self.deactivated_hook_enabled.value() {
                Some(self.deactivated_hook.text().to_string())
            } else {
                None
            },
        }
    }
}
