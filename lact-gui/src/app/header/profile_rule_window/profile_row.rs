use crate::app::{header::profile_rule_window::ProfileRuleWindowMsg, msg::AppMsg, APP_BROKER};
use gtk::{
    glib::GString,
    pango,
    prelude::{
        BoxExt, ButtonExt, CheckButtonExt, EditableExt, EntryBufferExt, EntryBufferExtManual,
        EntryExt, GridExt, OrientableExt, PopoverExt, SelectionModelExt, WidgetExt,
    },
};
use lact_schema::{ProcessInfo, ProcessProfileRule, ProfileRule, ProfileWatcherState};
use relm4::{
    binding::{BoolBinding, StringBinding},
    typed_view::list::{RelmListItem, TypedListView},
    view, RelmObjectExt, RelmWidgetExt,
};
use std::fmt::Write;
use tracing::debug;

const PROCESS_PAGE: &str = "process";
const GAMEMODE_PAGE: &str = "gamemode";

pub struct ProfileRuleRow {
    process_listview: TypedListView<ProcessListItem, gtk::SingleSelection>,

    selected_page: StringBinding,
    filter_by_args: BoolBinding,
    gamemode_filter_by_process: BoolBinding,

    process_search_filter: StringBinding,

    process_name_buffer: gtk::EntryBuffer,
    args_buffer: gtk::EntryBuffer,
}

#[derive(Debug)]
pub enum ProfileRuleRowMsg {
    Changed,
    WatcherState(ProfileWatcherState),
    ProcessFilterChanged(GString),
    SetFromSelectedProcess,
}

#[relm4::factory(pub)]
impl relm4::factory::FactoryComponent for ProfileRuleRow {
    type Init = ProfileRule;
    type Input = ProfileRuleRowMsg;
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
                set_markup: &format_rule(&self.get_configured_rule()),
            },

            gtk::MenuButton {
                set_icon_name: "open-menu-symbolic",
                set_tooltip: "Edit Rule",

                #[wrap(Some)]
                set_popover: main_popover = &gtk::Popover {
                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 10,
                        set_size_request: (300, 120),

                        gtk::StackSidebar {
                            set_stack: &stack,
                        },

                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,

                            #[name = "stack"]
                            gtk::Stack {
                                connect_visible_child_name_notify => ProfileRuleRowMsg::Changed,

                                add_titled[Some(PROCESS_PAGE), "A process is running"] = &gtk::Grid {
                                    set_row_spacing: 5,
                                    set_column_spacing: 5,

                                    attach[0, 0, 1, 1] = &gtk::Label {
                                        set_label: "Process Name:",
                                        set_halign: gtk::Align::Start,
                                    },

                                    attach[2, 0, 1, 1] = &gtk::Entry {
                                        set_buffer: &self.process_name_buffer,
                                        set_hexpand: true,
                                        set_placeholder_text: Some("Cyberpunk2077.exe"),
                                    },

                                    attach[3, 0, 1, 1] = &gtk::MenuButton {
                                        set_icon_name: "view-list-symbolic",

                                        #[wrap(Some)]
                                        set_popover: process_filter_popover = &gtk::Popover {
                                            gtk::Box {
                                                set_orientation: gtk::Orientation::Vertical,
                                                set_spacing: 5,

                                                gtk::SearchEntry {
                                                    connect_search_changed[sender] => move |entry| {
                                                        sender.input(ProfileRuleRowMsg::ProcessFilterChanged(entry.text()));
                                                    },
                                                },

                                                gtk::ScrolledWindow {
                                                    set_size_request: (400, 350),

                                                    self.process_listview.view.clone() -> gtk::ListView {
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
                                        connect_toggled => ProfileRuleRowMsg::Changed,
                                        add_binding: (&self.filter_by_args, "active"),
                                    },

                                    attach[2, 1, 2, 1]: args_entry = &gtk::Entry {
                                        set_buffer: &self.args_buffer,
                                        set_hexpand: true,
                                        set_sensitive: false,
                                        add_binding: (&self.filter_by_args, "sensitive"),
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
                                        connect_toggled => ProfileRuleRowMsg::Changed,
                                        add_binding: (&self.gamemode_filter_by_process, "active"),
                                    },

                                    attach[2, 0, 1, 1]: gamemode_process_name_entry = &gtk::Entry {
                                        set_buffer: &self.process_name_buffer,
                                        set_hexpand: true,
                                        set_placeholder_text: Some("Cyberpunk2077.exe"),
                                        set_sensitive: false,
                                        add_binding: (&self.gamemode_filter_by_process, "sensitive"),
                                    },

                                    attach[0, 1, 1, 1] = &gtk::Label {
                                        set_label: "Arguments Contain:",
                                        set_halign: gtk::Align::Start,
                                    },

                                    attach[1, 1, 1, 1]: gamemode_filter_by_args_checkbutton = &gtk::CheckButton {
                                        connect_toggled => ProfileRuleRowMsg::Changed,
                                        add_binding: (&self.filter_by_args, "active"),
                                    },

                                    attach[2, 1, 1, 1]: gamemode_args_entry = &gtk::Entry {
                                        set_buffer: &self.args_buffer,
                                        set_hexpand: true,
                                        set_sensitive: false,
                                        add_binding: (&self.filter_by_args, "sensitive"),
                                    },
                                },

                                add_binding: (&self.selected_page, "visible-child-name"),
                            },

                            gtk::Button {
                                set_label: "OK",
                                set_align: gtk::Align::End,
                                set_expand: true,
                                connect_clicked[main_popover] => move |_| {
                                    main_popover.popdown();
                                }
                            }
                        },
                    },
                },
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
        sender: relm4::FactorySender<Self>,
    ) -> Self {
        let process_name_buffer = gtk::EntryBuffer::default();
        let args_buffer = gtk::EntryBuffer::default();

        process_name_buffer.connect_text_notify({
            let sender = sender.clone();
            move |_| {
                sender.input(ProfileRuleRowMsg::Changed);
            }
        });
        args_buffer.connect_text_notify({
            let sender = sender.clone();
            move |_| {
                sender.input(ProfileRuleRowMsg::Changed);
            }
        });

        if let ProfileRule::Process(rule) | ProfileRule::Gamemode(Some(rule)) = &rule {
            process_name_buffer.set_text(rule.name.as_ref());
            args_buffer.set_text(rule.args.as_deref().unwrap_or_default());
        };

        let mut process_listview = TypedListView::<ProcessListItem, gtk::SingleSelection>::new();
        process_listview.selection_model.set_autoselect(false);

        let initial_page = match &rule {
            ProfileRule::Process(_) => PROCESS_PAGE,
            ProfileRule::Gamemode(_) => GAMEMODE_PAGE,
            _ => PROCESS_PAGE, // Fallback
        };
        let selected_page = StringBinding::new(initial_page);

        let filter_by_args = BoolBinding::new(args_buffer.length() > 0);
        let gamemode_filter_by_process = BoolBinding::new(process_name_buffer.length() > 0);

        for bool_bind in [&filter_by_args, &gamemode_filter_by_process] {
            bool_bind.connect_value_notify({
                let sender = sender.clone();
                move |_| {
                    sender.input(ProfileRuleRowMsg::Changed);
                }
            });
        }

        let process_search_filter = StringBinding::default();

        process_listview.add_filter({
            let process_filter = process_search_filter.clone();
            move |process| process.0.cmdline.contains(process_filter.value().as_str())
        });
        process_listview
            .selection_model
            .connect_selected_item_notify(move |_| {
                sender.input(ProfileRuleRowMsg::SetFromSelectedProcess);
            });

        Self {
            selected_page,
            process_name_buffer,
            gamemode_filter_by_process,
            process_search_filter,
            filter_by_args,
            args_buffer,
            process_listview,
        }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: Self::Input,
        sender: relm4::FactorySender<Self>,
    ) {
        match msg {
            ProfileRuleRowMsg::WatcherState(state) => {
                self.process_listview.clear();
                self.process_listview
                    .extend_from_iter(state.process_list.into_values().map(ProcessListItem).rev());
            }
            ProfileRuleRowMsg::ProcessFilterChanged(filter) => {
                self.process_search_filter.set_value(filter.as_str());

                self.process_listview.set_filter_status(0, false);
                if !filter.is_empty() {
                    self.process_listview.set_filter_status(0, true);
                }
            }
            ProfileRuleRowMsg::Changed => {
                sender.output(ProfileRuleWindowMsg::Evaluate).unwrap();
            }
            ProfileRuleRowMsg::SetFromSelectedProcess => {
                let index = self.process_listview.selection_model.selected();

                let filter_text = self.process_search_filter.value();
                let item = if filter_text.is_empty() {
                    self.process_listview.get(index)
                } else {
                    // Indexing is not aware of filters, so we have to apply the filter here to find a matching index
                    (0..self.process_listview.len())
                        .map(|i| self.process_listview.get(i).unwrap())
                        .filter(|item| item.borrow().0.cmdline.contains(filter_text.as_str()))
                        .nth(index as usize)
                };
                if let Some(item) = item {
                    let info = &item.borrow().0;
                    self.process_name_buffer.set_text(info.name.as_ref());
                    self.args_buffer.set_text(info.cmdline.as_ref());
                }

                self.process_listview.selection_model.unselect_all();

                widgets.process_filter_popover.popdown();
                widgets.main_popover.popdown();
            }
        }

        self.update_view(widgets, sender);
    }
}

impl ProfileRuleRow {
    pub fn get_configured_rule(&self) -> ProfileRule {
        let process_name = self.process_name_buffer.text();
        let process_args = self.args_buffer.text();

        match self.selected_page.value().as_str() {
            PROCESS_PAGE => {
                let args = if self.filter_by_args.value() {
                    Some(process_args.as_str().into())
                } else {
                    None
                };
                ProfileRule::Process(ProcessProfileRule {
                    name: process_name.as_str().into(),
                    args,
                })
            }
            GAMEMODE_PAGE => {
                let args = if self.filter_by_args.value() {
                    Some(process_args.as_str().into())
                } else {
                    None
                };
                let rule = if !self.gamemode_filter_by_process.value() && args.is_none() {
                    None
                } else {
                    Some(ProcessProfileRule {
                        name: process_name.as_str().into(),
                        args,
                    })
                };
                ProfileRule::Gamemode(rule)
            }
            _ => unreachable!(),
        }
    }
}

fn format_rule(rule: &ProfileRule) -> String {
    let mut text = String::new();

    match rule {
        ProfileRule::Process(process_rule) => {
            if !process_rule.name.is_empty() {
                write!(text, "Process <b>{}</b> is running", process_rule.name).unwrap();
            } else {
                write!(text, "Process is running <b>(unconfigured)</b>").unwrap();
            }
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
