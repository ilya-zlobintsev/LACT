use gtk::{
    glib::GStr,
    prelude::{EntryBufferExtManual, GtkWindowExt, OrientableExt, WidgetExt},
};
use lact_schema::{ProcessProfileRule, ProfileRule};
use relm4::{ComponentParts, ComponentSender, RelmWidgetExt};

const PROCESS_PAGE: &str = "process";
const GAMEMODE_PAGE: &str = "gamemode";

pub struct RuleWindow {
    process_name_buffer: gtk::EntryBuffer,
    args_buffer: gtk::EntryBuffer,
    rule: ProfileRule,
}

#[derive(Debug)]
pub enum RuleWindowMsg {}

#[relm4::component(pub)]
impl relm4::Component for RuleWindow {
    type Init = (Option<ProfileRule>, String);
    type Input = RuleWindowMsg;
    type Output = ();
    type CommandOutput = ();

    view! {
        gtk::Dialog {
            set_default_size: (600, 300),
            set_title: Some("Profile activation rules"),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                gtk::Label {
                    set_markup: &format!("<span font_desc='11'><b>Activate profile '{name}' when:</b></span>"),
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
                        add_titled[Some(PROCESS_PAGE), "A process is running"] = &gtk::Box {

                        },

                        add_titled[Some(GAMEMODE_PAGE), "Gamemode is active"] = &gtk::Box {

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
        (init, name): Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let rule = init.unwrap_or_else(|| ProfileRule::Process(ProcessProfileRule::default()));

        let model = Self {
            rule,
            process_name_buffer: gtk::EntryBuffer::new(GStr::NONE),
            args_buffer: gtk::EntryBuffer::new(GStr::NONE),
        };

        if let ProfileRule::Process(rule) | ProfileRule::Gamemode(Some(rule)) = &model.rule {
            model.process_name_buffer.set_text(&rule.name);
            model.args_buffer.set_text(&rule.name);
        }

        let widgets = view_output!();

        root.present();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, root: &Self::Root) {
        match msg {}
    }
}
