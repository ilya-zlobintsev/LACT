use std::time::Duration;

use crate::app::{msg::AppMsg, APP_BROKER};
use gtk::{
    glib::{GStr, GString},
    prelude::{
        BoxExt, CheckButtonExt, DialogExt, DialogExtManual, EditableExt, EntryBufferExtManual,
        EntryExt, GridExt, GtkWindowExt, ObjectExt, OrientableExt, PopoverExt, SelectionModelExt,
        WidgetExt,
    },
    SingleSelection,
};
use lact_schema::{ProcessInfo, ProcessProfileRule, ProfileRule, ProfileWatcherState};
use relm4::{
    tokio::time::sleep,
    typed_view::list::{RelmListItem, TypedListView},
    view, ComponentParts, ComponentSender, RelmWidgetExt,
};
use tracing::debug;

const EVALUATE_INTERVAL_MS: u64 = 250;

const PROCESS_PAGE: &str = "process";
const GAMEMODE_PAGE: &str = "gamemode";

pub struct ProfileRuleWindow {
    profile_name: String,
    process_name_buffer: gtk::EntryBuffer,
    args_buffer: gtk::EntryBuffer,
    rule: ProfileRule,
    process_list_view: TypedListView<ProcessListItem, SingleSelection>,
    currently_matches: bool,
}

#[derive(Debug)]
pub enum ProfileRuleWindowMsg {
    Show {
        profile_name: String,
        rule: ProfileRule,
    },
    ProcessFilterChanged(GString),
    WatcherState(ProfileWatcherState),
    SetFromSelectedProcess,
    Evaluate,
    EvaluationResult(bool),
    Save,
}

#[relm4::component(pub)]
impl relm4::Component for ProfileRuleWindow {
    type Init = ();
    type Input = ProfileRuleWindowMsg;
    type Output = ();
    type CommandOutput = ();

    view! {
        gtk::Dialog {
            set_default_size: (600, 300),
            set_title: Some("Profile activation rules"),
            set_hide_on_close: true,
            connect_response[root, sender] => move |_, response| {
                match response {
                    gtk::ResponseType::Accept => {
                        sender.input(ProfileRuleWindowMsg::Save);
                        root.hide();
                    }
                    gtk::ResponseType::Cancel => root.hide(),
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

                                        connect_visible_notify => |_| {
                                            debug!("requesting profile watcher state");
                                            APP_BROKER.send(AppMsg::ReloadProfiles { include_state: true });
                                        },
                                    },
                                },

                                attach[0, 1, 1, 1] = &gtk::Label {
                                    set_label: "Arguments Contain:",
                                    set_halign: gtk::Align::Start,
                                },


                                attach[1, 1, 1, 1]: filter_by_args_checkbutton = &gtk::CheckButton {
                                    connect_toggled => ProfileRuleWindowMsg::Evaluate,
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
                                },

                                attach[2, 1, 1, 1]: gamemode_args_entry = &gtk::Entry {
                                    set_buffer: &model.args_buffer,
                                    set_hexpand: true,
                                    set_sensitive: false,
                                    connect_changed => ProfileRuleWindowMsg::Evaluate,
                                },
                            },

                            set_visible_child_name: match &model.rule {
                                ProfileRule::Process(_) => PROCESS_PAGE,
                                ProfileRule::Gamemode(_) => GAMEMODE_PAGE,
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
        (): Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let task_sender = sender.clone();
        relm4::spawn_local(async move {
            loop {
                sleep(Duration::from_millis(EVALUATE_INTERVAL_MS)).await;
                task_sender.input(ProfileRuleWindowMsg::Evaluate);
            }
        });

        let mut model = Self {
            rule: ProfileRule::default(),
            profile_name: String::new(),
            process_name_buffer: gtk::EntryBuffer::new(GStr::NONE),
            args_buffer: gtk::EntryBuffer::new(GStr::NONE),
            process_list_view: TypedListView::new(),
            currently_matches: false,
        };

        model
            .process_list_view
            .selection_model
            .set_autoselect(false);

        let process_listview = &model.process_list_view.view;
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
            .bidirectional()
            .build();

        widgets
            .gamemode_filter_by_process_checkbutton
            .bind_property("active", &widgets.gamemode_process_name_entry, "sensitive")
            .bidirectional()
            .build();

        widgets
            .gamemode_filter_by_args_checkbutton
            .bind_property("active", &widgets.gamemode_args_entry, "sensitive")
            .bidirectional()
            .build();

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
            ProfileRuleWindowMsg::Show { profile_name, rule } => {
                self.profile_name = profile_name;

                let page = match rule {
                    ProfileRule::Process(rule) => {
                        self.process_name_buffer.set_text(rule.name.as_ref());
                        self.args_buffer
                            .set_text(rule.args.as_deref().unwrap_or_default());
                        PROCESS_PAGE
                    }
                    ProfileRule::Gamemode(Some(rule)) => {
                        self.process_name_buffer.set_text(rule.name.as_ref());
                        self.args_buffer
                            .set_text(rule.args.as_deref().unwrap_or_default());
                        GAMEMODE_PAGE
                    }
                    ProfileRule::Gamemode(None) => {
                        self.process_name_buffer.set_text("");
                        self.args_buffer.set_text("");
                        GAMEMODE_PAGE
                    }
                };
                widgets.stack.set_visible_child_name(page);

                widgets
                    .filter_by_args_checkbutton
                    .set_active(self.args_buffer.length() > 0);
                widgets
                    .gamemode_filter_by_process_checkbutton
                    .set_active(self.process_name_buffer.length() > 0);
                widgets
                    .gamemode_filter_by_args_checkbutton
                    .set_active(self.args_buffer.length() > 0);

                root.present();
            }
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
                    APP_BROKER.send(AppMsg::EvaluateProfile(rule));
                }
            }
            ProfileRuleWindowMsg::EvaluationResult(matches) => {
                self.currently_matches = matches;
            }
            ProfileRuleWindowMsg::Save => {
                APP_BROKER.send(AppMsg::SetProfileRule {
                    name: self.profile_name.clone(),
                    rule: Some(self.get_rule(widgets)),
                });
            }
        }

        self.update_view(widgets, sender);
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
