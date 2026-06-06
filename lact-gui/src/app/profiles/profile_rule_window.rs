pub mod profile_rule_row;

use crate::I18N;
use crate::app::{APP_BROKER, msg::AppMsg};
use adw::prelude::*;
use gtk::pango;
use i18n_embed_fl::fl;
use lact_schema::{ProfileRule, config::ProfileHooks};
use profile_rule_row::ProfileRuleRow;
use relm4::{
    ComponentParts, ComponentSender, RelmObjectExt, RelmWidgetExt,
    binding::BoolBinding,
    css,
    prelude::{DynamicIndex, FactoryVecDeque},
    tokio::time::sleep,
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
    pub parent: gtk::Widget,
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
        adw::Dialog {
            set_content_width: 640,
            set_follows_content_size: true,
            set_title: &fl!(I18N, "profile-rules"),

            #[wrap(Some)]
            set_child = &adw::ToolbarView {
                add_top_bar = &adw::HeaderBar {
                    #[wrap(Some)]
                    set_title_widget = &gtk::StackSwitcher {
                        set_stack: Some(&stack),
                    },
                },

                #[name = "stack"]
                #[wrap(Some)]
                set_content = &gtk::Stack {
                    add_titled[None, &fl!(I18N, "profile-activation")] = &adw::PreferencesPage {

                        add = &adw::PreferencesGroup {
                            set_title: &fl!(I18N, "profile-activation-desc", name = model.profile_name.as_str()),

                            gtk::ListBox {
                                set_selection_mode: gtk::SelectionMode::None,
                                add_css_class: css::BOXED_LIST,

                                #[name = "multi_or_checkbutton"]
                                gtk::CheckButton {
                                    set_label: Some(&fl!(I18N, "any-rules-matched")),
                                    set_active: !matches!(rule, ProfileRule::And(_)),
                                    set_margin_all: 10,
                                    connect_toggled => ProfileRuleWindowMsg::Evaluate,
                                },

                                #[name = "multi_and_checkbutton"]
                                gtk::CheckButton {
                                    set_label: Some(&fl!(I18N, "all-rules-matched")),
                                    set_group: Some(&multi_or_checkbutton),
                                    set_active: matches!(rule, ProfileRule::And(_)),
                                    set_margin_all: 10,
                                    connect_toggled => ProfileRuleWindowMsg::Evaluate,
                                },
                            },
                        },

                        add = &adw::PreferencesGroup {
                            set_title: &fl!(I18N, "profile-rules"),

                            #[wrap(Some)]
                            set_header_suffix = &gtk::Button {
                                set_icon_name: "list-add-symbolic",
                                set_tooltip: &fl!(I18N, "profile-rules"),
                                add_css_class: "flat",
                                connect_clicked => ProfileRuleWindowMsg::AddSubrule,
                            },

                            #[local_ref]
                            sub_rules_listview -> gtk::ListBox {
                                set_selection_mode: gtk::SelectionMode::None,
                                add_css_class: css::BOXED_LIST,
                            },
                        },

                        add = &adw::PreferencesGroup {
                            gtk::Box {
                                set_orientation: gtk::Orientation::Horizontal,
                                set_spacing: 5,

                                gtk::Image {
                                    #[watch]
                                    set_icon_name: match model.currently_matches {
                                        true => Some("object-select-symbolic"),
                                        false => Some("list-remove-symbolic"),
                                    },
                                },

                                gtk::Label {
                                    #[watch]
                                    set_markup: &if model.auto_switch {
                                        fl!(I18N, "activation-settings-status", matched = model.currently_matches.to_string())
                                    } else {
                                        format!(
                                            "<b>{}</b>",
                                            fl!(I18N, "activation-auto-switching-disabled")
                                        )
                                    },
                                },
                            },
                        },
                    },

                    add_titled[None, &fl!(I18N, "profile-hooks")] = &adw::PreferencesPage {

                        add = &adw::PreferencesGroup {
                            set_description: Some(&fl!(I18N, "profile-hook-command", cmd = model.profile_name.as_str())),

                            gtk::ListBox {
                                set_selection_mode: gtk::SelectionMode::None,
                                add_css_class: css::BOXED_LIST,

                                gtk::ListBoxRow {
                                    gtk::Box {
                                        set_orientation: gtk::Orientation::Horizontal,
                                        set_spacing: 10,
                                        set_margin_all: 10,

                                        gtk::CheckButton {
                                            set_label: Some(&fl!(I18N, "profile-hook-activated")),
                                            add_binding: (&model.activated_hook_enabled, "active"),
                                            set_size_group: &hook_command_size_group,
                                        },

                                        gtk::Entry {
                                            add_binding: (&model.activated_hook_enabled, "sensitive"),
                                            set_buffer: &model.activated_hook,
                                            set_hexpand: true,
                                        },
                                    },
                                },

                                gtk::ListBoxRow {
                                    gtk::Box {
                                        set_orientation: gtk::Orientation::Horizontal,
                                        set_spacing: 10,
                                        set_margin_all: 10,

                                        gtk::CheckButton {
                                            set_label: Some(&fl!(I18N, "profile-hook-deactivated")),
                                            add_binding: (&model.deactivated_hook_enabled, "active"),
                                            set_size_group: &hook_command_size_group,
                                        },

                                        gtk::Entry {
                                            add_binding: (&model.deactivated_hook_enabled, "sensitive"),
                                            set_buffer: &model.deactivated_hook,
                                            set_hexpand: true,
                                        },
                                    },
                                },
                            },
                        },

                        add = &adw::PreferencesGroup {
                            gtk::Box {
                                set_orientation: gtk::Orientation::Horizontal,
                                set_spacing: 5,

                                gtk::Image {
                                    set_icon_name: Some("dialog-warning-symbolic"),
                                },

                                gtk::Label {
                                    set_label: &fl!(I18N, "profile-hook-note"),
                                    set_wrap: true,
                                    set_wrap_mode: pango::WrapMode::Word,
                                    add_css_class: "caption-heading",
                                    set_hexpand: true,
                                },
                            }
                        },
                    },
                },

                add_bottom_bar = &gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 10,
                    set_margin_all: 10,
                    set_halign: gtk::Align::End,

                    gtk::Button {
                        set_label: &fl!(I18N, "cancel"),
                        connect_clicked[root = root.downgrade()] => move |_| {
                            if let Some(root) = root.upgrade() {
                                root.close();
                            }
                        },
                    },

                    gtk::Button {
                        set_label: &fl!(I18N, "save"),
                        add_css_class: css::SUGGESTED_ACTION,
                        connect_clicked => ProfileRuleWindowMsg::Save,
                    },
                },
            }
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
            parent,
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

        root.present(Some(&parent));

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
                root.close();
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
