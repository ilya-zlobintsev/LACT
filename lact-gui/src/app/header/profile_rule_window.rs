use crate::app::{msg::AppMsg, APP_BROKER};
use gtk::{
    glib::{GStr, GString},
    pango,
    prelude::{
        BoxExt, ButtonExt, CheckButtonExt, DialogExt, DialogExtManual, EditableExt,
        EntryBufferExtManual, EntryExt, GridExt, GtkWindowExt, ObjectExt, OrientableExt,
        PopoverExt, SelectionModelExt, WidgetExt,
    },
};
use lact_schema::{ProcessInfo, ProcessProfileRule, ProfileRule, ProfileWatcherState};
use relm4::{
    prelude::{DynamicIndex, FactoryVecDeque},
    tokio::time::sleep,
    typed_view::list::{RelmListItem, TypedListView},
    view, ComponentParts, ComponentSender, RelmWidgetExt,
};
use std::{fmt::Write, time::Duration};
use tracing::debug;

const EVALUATE_INTERVAL_MS: u64 = 250;

const PROCESS_PAGE: &str = "process";
const GAMEMODE_PAGE: &str = "gamemode";
const MULTI_RULE_PAGE: &str = "multiple";

pub struct ProfileRuleWindow {
    profile_name: String,
    process_name_buffer: gtk::EntryBuffer,
    args_buffer: gtk::EntryBuffer,
    rule: ProfileRule,
    process_list_view: TypedListView<ProcessListItem, gtk::SingleSelection>,
    sub_rules_list_view: FactoryVecDeque<MultiRuleRow>,
    currently_matches: bool,
}

#[derive(Debug)]
pub enum ProfileRuleWindowMsg {
    ProcessFilterChanged(GString),
    WatcherState(ProfileWatcherState),
    SetFromSelectedProcess,
    Evaluate,
    EvaluationResult(bool),
    AddSubrule,
    EditSubrule(DynamicIndex),
    RemoveSubrule(DynamicIndex),
    Save,
}

#[derive(Debug)]
pub enum CommandOutput {
    SubruleAdded(ProfileRule),
    SubruleEdited(ProfileRule, DynamicIndex),
}

#[relm4::component(pub)]
impl relm4::Component for ProfileRuleWindow {
    type Init = (String, ProfileRule);
    type Input = ProfileRuleWindowMsg;
    type Output = (String, ProfileRule);
    type CommandOutput = Option<CommandOutput>;

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

                    gtk::StackSidebar {
                        set_stack: &stack,
                    },

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_margin_all: 10,
                        set_spacing: 10,

                        #[name = "stack"]
                        gtk::Stack {
                            connect_visible_child_name_notify => ProfileRuleWindowMsg::Evaluate,

                            add_titled[Some(PROCESS_PAGE), "A process is running"] = &gtk::Grid {
                                set_row_spacing: 5,
                                set_column_spacing: 5,

                                attach[0, 0, 1, 1] = &gtk::Label {
                                    set_label: "Process Name:",
                                    set_halign: gtk::Align::Start,
                                },

                                attach[2, 0, 1, 1] = &gtk::Entry {
                                    set_buffer: &model.process_name_buffer,
                                    set_hexpand: true,
                                    set_placeholder_text: Some("Cyberpunk2077.exe"),
                                    connect_changed => ProfileRuleWindowMsg::Evaluate,
                                },

                                attach[3, 0, 1, 1] = &gtk::MenuButton {
                                    set_icon_name: "view-list-symbolic",

                                    #[wrap(Some)]
                                    set_popover: process_filter_popover = &gtk::Popover {
                                        gtk::Box {
                                            set_orientation: gtk::Orientation::Vertical,
                                            set_spacing: 5,

                                            #[name = "process_search_entry"]
                                            gtk::SearchEntry {
                                                connect_search_changed[sender] => move |entry| {
                                                    sender.input(ProfileRuleWindowMsg::ProcessFilterChanged(entry.text()));
                                                },
                                            },

                                            gtk::ScrolledWindow {
                                                set_size_request: (400, 350),

                                                #[local_ref]
                                                process_listview -> gtk::ListView {
                                                    set_show_separators: true,
                                                },
                                            }
                                        },

                                        connect_visible_notify[sender] => move |_| {
                                            debug!("requesting profile watcher state");
                                            APP_BROKER.send(AppMsg::ReloadProfiles { state_sender: Some(sender.input_sender().clone())});
                                        },
                                    },
                                },

                                attach[0, 1, 1, 1] = &gtk::Label {
                                    set_label: "Arguments Contain:",
                                    set_halign: gtk::Align::Start,
                                },


                                attach[1, 1, 1, 1]: filter_by_args_checkbutton = &gtk::CheckButton {
                                    connect_toggled => ProfileRuleWindowMsg::Evaluate,
                                    set_active: model.args_buffer.length() > 0,
                                },

                                attach[2, 1, 1, 1]: args_entry = &gtk::Entry {
                                    set_buffer: &model.args_buffer,
                                    set_hexpand: true,
                                    set_sensitive: false,
                                    connect_changed => ProfileRuleWindowMsg::Evaluate,
                                },
                            },

                            add_titled[Some(GAMEMODE_PAGE), "Gamemode is active"] = &gtk::Grid {
                                set_row_spacing: 5,
                                set_column_spacing: 10,

                                attach[0, 0, 1, 1] = &gtk::Label {
                                    set_label: "With a specific process:",
                                    set_halign: gtk::Align::Start,
                                },

                                attach[1, 0, 1, 1]: gamemode_filter_by_process_checkbutton = &gtk::CheckButton {
                                    connect_toggled => ProfileRuleWindowMsg::Evaluate,
                                    set_active: model.process_name_buffer.length() > 0,
                                },

                                attach[2, 0, 1, 1]: gamemode_process_name_entry = &gtk::Entry {
                                    set_buffer: &model.process_name_buffer,
                                    set_hexpand: true,
                                    set_placeholder_text: Some("Cyberpunk2077.exe"),
                                    set_sensitive: false,
                                    connect_changed => ProfileRuleWindowMsg::Evaluate,
                                },

                                attach[0, 1, 1, 1] = &gtk::Label {
                                    set_label: "Arguments Contain:",
                                    set_halign: gtk::Align::Start,
                                },

                                attach[1, 1, 1, 1]: gamemode_filter_by_args_checkbutton = &gtk::CheckButton {
                                    connect_toggled => ProfileRuleWindowMsg::Evaluate,
                                    set_active: model.args_buffer.length() > 0,
                                },

                                attach[2, 1, 1, 1]: gamemode_args_entry = &gtk::Entry {
                                    set_buffer: &model.args_buffer,
                                    set_hexpand: true,
                                    set_sensitive: false,
                                    connect_changed => ProfileRuleWindowMsg::Evaluate,
                                },
                            },

                            add_titled[Some(MULTI_RULE_PAGE), "Multiple Rules"] = &gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_spacing: 5,

                                #[name = "multi_or_checkbutton"]
                                gtk::CheckButton {
                                    set_label: Some("Any of the following rules are matched:"),
                                    set_active: !matches!(model.rule, ProfileRule::And(_)),
                                },

                                #[name = "multi_and_checkbutton"]
                                gtk::CheckButton {
                                    set_label: Some("All of the following rules are matched:"),
                                    set_group: Some(&multi_or_checkbutton),
                                    set_active: matches!(model.rule, ProfileRule::And(_)),
                                },

                                gtk::Separator {},

                                #[local_ref]
                                sub_rules_listview -> gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,
                                    set_spacing: 5,
                                },

                                gtk::Box {
                                    set_orientation: gtk::Orientation::Horizontal,
                                    set_spacing: 5,

                                    gtk::Button {
                                        set_hexpand: true,
                                        set_icon_name: "list-add-symbolic",
                                        connect_clicked => ProfileRuleWindowMsg::AddSubrule,
                                    },
                                },
                            },

                            set_visible_child_name: match &model.rule {
                                ProfileRule::Process(_) => PROCESS_PAGE,
                                ProfileRule::Gamemode(_) => GAMEMODE_PAGE,
                                ProfileRule::And(_) | ProfileRule::Or(_) => MULTI_RULE_PAGE,
                            }
                        },

                        gtk::Separator {},

                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 5,

                            gtk::Label {
                                #[watch]
                                set_markup: &format!(
                                    "Selected settings are currently <b>{}</b>",
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

        let sub_rules_list_view = FactoryVecDeque::builder()
            .launch_default()
            .forward(sender.input_sender(), |msg| msg);

        let mut model = Self {
            rule,
            profile_name,
            process_name_buffer: gtk::EntryBuffer::new(GStr::NONE),
            args_buffer: gtk::EntryBuffer::new(GStr::NONE),
            process_list_view: TypedListView::new(),
            sub_rules_list_view,
            currently_matches: false,
        };

        model
            .process_list_view
            .selection_model
            .set_autoselect(false);

        match &model.rule {
            ProfileRule::Process(rule) | ProfileRule::Gamemode(Some(rule)) => {
                model.process_name_buffer.set_text(rule.name.as_ref());
                model
                    .args_buffer
                    .set_text(rule.args.as_deref().unwrap_or_default());
            }
            ProfileRule::Gamemode(None) => (),
            ProfileRule::And(subrules) | ProfileRule::Or(subrules) => {
                for rule in subrules.iter().cloned() {
                    model.sub_rules_list_view.guard().push_back(rule);
                }
            }
        };

        let process_listview = &model.process_list_view.view;
        let sub_rules_listview = model.sub_rules_list_view.widget();
        let widgets = view_output!();

        model.process_list_view.add_filter({
            let search_entry = widgets.process_search_entry.clone();
            move |process| process.0.cmdline.contains(search_entry.text().as_str())
        });
        model
            .process_list_view
            .selection_model
            .connect_selected_item_notify(move |_| {
                sender.input(ProfileRuleWindowMsg::SetFromSelectedProcess);
            });

        widgets
            .filter_by_args_checkbutton
            .bind_property("active", &widgets.args_entry, "sensitive")
            .sync_create()
            .bidirectional()
            .build();

        widgets
            .gamemode_filter_by_process_checkbutton
            .bind_property("active", &widgets.gamemode_process_name_entry, "sensitive")
            .sync_create()
            .bidirectional()
            .build();

        widgets
            .gamemode_filter_by_args_checkbutton
            .bind_property("active", &widgets.gamemode_args_entry, "sensitive")
            .sync_create()
            .bidirectional()
            .build();

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
            ProfileRuleWindowMsg::ProcessFilterChanged(filter) => {
                self.process_list_view.set_filter_status(0, false);
                if !filter.is_empty() {
                    self.process_list_view.set_filter_status(0, true);
                }
            }
            ProfileRuleWindowMsg::WatcherState(state) => {
                self.process_list_view.clear();
                self.process_list_view
                    .extend_from_iter(state.process_list.into_values().map(ProcessListItem).rev());
            }
            ProfileRuleWindowMsg::SetFromSelectedProcess => {
                let index = self.process_list_view.selection_model.selected();

                let filter_text = widgets.process_search_entry.text();
                let item = if filter_text.is_empty() {
                    self.process_list_view.get(index)
                } else {
                    // Indexing is not aware of filters, so we have to apply the filter here to find a matching index
                    (0..self.process_list_view.len())
                        .map(|i| self.process_list_view.get(i).unwrap())
                        .filter(|item| item.borrow().0.cmdline.contains(filter_text.as_str()))
                        .nth(index as usize)
                };
                if let Some(item) = item {
                    let info = &item.borrow().0;
                    self.process_name_buffer.set_text(info.name.as_ref());
                    self.args_buffer.set_text(info.cmdline.as_ref());
                }

                self.process_list_view.selection_model.unselect_all();
                widgets.process_filter_popover.popdown();
            }
            ProfileRuleWindowMsg::Evaluate => {
                if root.is_visible() {
                    let rule = self.get_rule(widgets);
                    APP_BROKER.send(AppMsg::EvaluateProfile(rule, sender.input_sender().clone()));
                }
            }
            ProfileRuleWindowMsg::AddSubrule => {
                let stream = ProfileRuleWindow::builder()
                    .launch(("Subrule".to_owned(), ProfileRule::default()))
                    .into_stream();

                sender.oneshot_command(async move {
                    stream
                        .recv_one()
                        .await
                        .map(|(_, rule)| CommandOutput::SubruleAdded(rule))
                });
            }
            ProfileRuleWindowMsg::EditSubrule(index) => {
                let current_rule = self
                    .sub_rules_list_view
                    .get(index.current_index())
                    .unwrap()
                    .rule
                    .clone();

                let stream = ProfileRuleWindow::builder()
                    .launch(("Subrule".to_owned(), current_rule))
                    .into_stream();

                sender.oneshot_command(async move {
                    stream
                        .recv_one()
                        .await
                        .map(|(_, rule)| CommandOutput::SubruleEdited(rule, index))
                });
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

    fn update_cmd(
        &mut self,
        msg: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        if let Some(msg) = msg {
            match msg {
                CommandOutput::SubruleAdded(rule) => {
                    self.sub_rules_list_view.guard().push_back(rule);
                }
                CommandOutput::SubruleEdited(rule, index) => {
                    self.sub_rules_list_view.send(index.current_index(), rule);
                }
            }
        }
    }
}

impl ProfileRuleWindow {
    fn get_rule(&self, widgets: &ProfileRuleWindowWidgets) -> ProfileRule {
        let process_name = self.process_name_buffer.text();
        let process_args = self.args_buffer.text();

        match widgets.stack.visible_child_name().as_deref() {
            Some(PROCESS_PAGE) => {
                let args = if widgets.filter_by_args_checkbutton.is_active() {
                    Some(process_args.as_str().into())
                } else {
                    None
                };
                ProfileRule::Process(ProcessProfileRule {
                    name: process_name.as_str().into(),
                    args,
                })
            }
            Some(GAMEMODE_PAGE) => {
                let args = if widgets.gamemode_filter_by_args_checkbutton.is_active() {
                    Some(process_args.as_str().into())
                } else {
                    None
                };
                let rule = if !widgets.gamemode_filter_by_process_checkbutton.is_active()
                    && args.is_none()
                {
                    None
                } else {
                    Some(ProcessProfileRule {
                        name: process_name.as_str().into(),
                        args,
                    })
                };
                ProfileRule::Gamemode(rule)
            }
            Some(MULTI_RULE_PAGE) => {
                let mut subrules = vec![];

                for i in 0..self.sub_rules_list_view.len() {
                    let rule = self.sub_rules_list_view.get(i).unwrap();
                    subrules.push(rule.rule.clone());
                }

                if widgets.multi_or_checkbutton.is_active() {
                    ProfileRule::Or(subrules)
                } else {
                    ProfileRule::And(subrules)
                }
            }
            _ => unreachable!(),
        }
    }
}

struct ProcessListItem(ProcessInfo);

struct ProcessListItemWidgets {
    label: gtk::Label,
}

impl RelmListItem for ProcessListItem {
    type Root = gtk::Box;
    type Widgets = ProcessListItemWidgets;

    fn setup(_list_item: &gtk::ListItem) -> (Self::Root, Self::Widgets) {
        view! {
            root_box = gtk::Box {
                #[name = "label"]
                gtk::Label {
                    set_halign: gtk::Align::Start,
                    set_hexpand: true,
                    set_selectable: false,
                },
            }
        }

        let widgets = ProcessListItemWidgets { label };
        (root_box, widgets)
    }

    fn bind(&mut self, widgets: &mut Self::Widgets, _root: &mut Self::Root) {
        let text = format!("<b>{}</b> ({})", self.0.name, self.0.cmdline);
        widgets.label.set_markup(&text);
    }
}

struct MultiRuleRow {
    rule: ProfileRule,
}

#[relm4::factory]
impl relm4::factory::FactoryComponent for MultiRuleRow {
    type Init = ProfileRule;
    type Input = ProfileRule;
    type Output = ProfileRuleWindowMsg;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        gtk::Box {
            gtk::Label {
                set_hexpand: true,
                set_halign: gtk::Align::Start,
                set_ellipsize: pango::EllipsizeMode::End,
                #[watch]
                set_markup: &format_rule(&self.rule),
            },

            gtk::Button {
                set_icon_name: "open-menu-symbolic",
                set_tooltip: "Edit Rule",
                connect_clicked[sender, index] => move |_| {
                    let _ = sender.output(ProfileRuleWindowMsg::EditSubrule(index.clone()));
                }
            },

            gtk::Button {
                set_icon_name: "list-remove-symbolic",
                set_tooltip: "Remove Rule",
                connect_clicked[sender, index] => move |_| {
                    let _ = sender.output(ProfileRuleWindowMsg::RemoveSubrule(index.clone()));
                }
            },
        }
    }

    fn init_model(
        rule: Self::Init,
        _index: &Self::Index,
        _sender: relm4::FactorySender<Self>,
    ) -> Self {
        Self { rule }
    }

    fn update(&mut self, rule: Self::Input, _sender: relm4::FactorySender<Self>) {
        self.rule = rule;
    }
}

fn format_rule(rule: &ProfileRule) -> String {
    let mut text = String::new();

    match rule {
        ProfileRule::Process(process_rule) => {
            write!(text, "Process <b>{}</b> is running", process_rule.name).unwrap();
            if let Some(args) = &process_rule.args {
                write!(text, " with args <b>{args}</b>").unwrap();
            }
        }
        ProfileRule::Gamemode(process_rule) => {
            write!(text, "Gamemode is active").unwrap();
            if let Some(process_rule) = process_rule {
                write!(text, "with process <b>{}</b>", process_rule.name).unwrap();
                if let Some(args) = &process_rule.args {
                    write!(text, " and args <b>{args}</b>").unwrap();
                }
            }
        }
        ProfileRule::And(subrules) => {
            write!(text, "All of the following rules are matched: ").unwrap();
            for (i, rule) in subrules.iter().enumerate() {
                if i > 0 {
                    text.push_str(", ");
                }
                text.push_str(&format_rule(rule));
            }
        }
        ProfileRule::Or(subrules) => {
            write!(text, "Any of the following rules are matched: ").unwrap();
            for (i, rule) in subrules.iter().enumerate() {
                if i > 0 {
                    text.push_str(", ");
                }
                text.push_str(&format_rule(rule));
            }
        }
    }

    text
}
