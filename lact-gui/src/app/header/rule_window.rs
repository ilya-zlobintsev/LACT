use gtk::{
    glib::GStr,
    prelude::{
        CheckButtonExt, EntryBufferExtManual, EntryExt, GridExt, GtkWindowExt, ObjectExt,
        OrientableExt, WidgetExt,
    },
};
use lact_schema::ProfileRule;
use relm4::{ComponentParts, ComponentSender, RelmWidgetExt};

const PROCESS_PAGE: &str = "process";
const GAMEMODE_PAGE: &str = "gamemode";

pub struct RuleWindow {
    profile_name: String,
    process_name_buffer: gtk::EntryBuffer,
    args_buffer: gtk::EntryBuffer,
    rule: ProfileRule,
}

#[derive(Debug)]
pub enum RuleWindowMsg {
    Show {
        profile_name: String,
        rule: ProfileRule,
    },
}

#[relm4::component(pub)]
impl relm4::Component for RuleWindow {
    type Init = ();
    type Input = RuleWindowMsg;
    type Output = ();
    type CommandOutput = ();

    view! {
        gtk::Dialog {
            set_default_size: (600, 300),
            set_title: Some("Profile activation rules"),
            set_hide_on_close: true,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

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

                    #[name = "stack"]
                    gtk::Stack {
                        add_titled[Some(PROCESS_PAGE), "A process is running"] = &gtk::Grid {
                            set_row_spacing: 5,
                            set_column_spacing: 10,
                            set_margin_all: 10,

                            attach[0, 0, 1, 1] = &gtk::Label {
                                set_label: "Process Name:",
                                set_halign: gtk::Align::Start,
                            },

                            attach[2, 0, 1, 1] = &gtk::Entry {
                                set_buffer: &model.process_name_buffer,
                                set_hexpand: true,
                                set_placeholder_text: Some("Cyberpunk2077.exe"),
                            },

                            attach[0, 1, 1, 1] = &gtk::Label {
                                set_label: "Arguments Contain:",
                                set_halign: gtk::Align::Start,
                            },


                            attach[1, 1, 1, 1]: filter_by_args_checkbutton = &gtk::CheckButton {
                                #[watch]
                                set_active: model.args_buffer.length() > 0,
                            },

                            attach[2, 1, 1, 1]: args_entry = &gtk::Entry {
                                set_buffer: &model.args_buffer,
                                set_hexpand: true,
                                set_sensitive: false,
                            },
                        },

                        add_titled[Some(GAMEMODE_PAGE), "Gamemode is active"] = &gtk::Grid {
                            set_row_spacing: 5,
                            set_column_spacing: 10,
                            set_margin_all: 10,

                            attach[0, 0, 1, 1] = &gtk::Label {
                                set_label: "With a specific process:",
                                set_halign: gtk::Align::Start,
                            },

                            attach[1, 0, 1, 1]: gamemode_filter_by_process_checkbutton = &gtk::CheckButton {
                                #[watch]
                                set_active: model.process_name_buffer.length() > 0,
                            },

                            attach[2, 0, 1, 1]: gamemode_process_name_entry = &gtk::Entry {
                                set_buffer: &model.process_name_buffer,
                                set_hexpand: true,
                                set_placeholder_text: Some("Cyberpunk2077.exe"),
                                set_sensitive: false,
                            },

                            attach[0, 1, 1, 1] = &gtk::Label {
                                set_label: "Arguments Contain:",
                                set_halign: gtk::Align::Start,
                            },

                            attach[1, 1, 1, 1]: gamemode_filter_by_args_checkbutton = &gtk::CheckButton {
                                #[watch]
                                set_active: model.args_buffer.length() > 0,
                            },

                            attach[2, 1, 1, 1]: gamemode_args_entry = &gtk::Entry {
                                set_buffer: &model.args_buffer,
                                set_hexpand: true,
                                set_sensitive: false,
                            },
                        },

                        set_visible_child_name: match &model.rule {
                            ProfileRule::Process(_) => PROCESS_PAGE,
                            ProfileRule::Gamemode(_) => GAMEMODE_PAGE,
                        }
                    },
                }

            }
        }
    }

    fn init(
        (): Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            rule: ProfileRule::default(),
            profile_name: String::new(),
            process_name_buffer: gtk::EntryBuffer::new(GStr::NONE),
            args_buffer: gtk::EntryBuffer::new(GStr::NONE),
        };

        let widgets = view_output!();

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

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>, root: &Self::Root) {
        match msg {
            RuleWindowMsg::Show { profile_name, rule } => {
                self.profile_name = profile_name;

                if let ProfileRule::Process(rule) | ProfileRule::Gamemode(Some(rule)) = rule {
                    self.process_name_buffer.set_text(&rule.name);
                    self.args_buffer.set_text(&rule.name);
                }

                root.present();
            }
        }
    }
}
