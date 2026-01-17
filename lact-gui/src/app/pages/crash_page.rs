use crate::app::msg::AppMsg;
use gtk::prelude::*;
use relm4::{ComponentParts, ComponentSender};

pub struct CrashPage {
    message: String,
}

#[relm4::component(pub)]
impl relm4::SimpleComponent for CrashPage {
    type Init = String;
    type Input = String;
    type Output = AppMsg;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 20,
            set_valign: gtk::Align::Center,
            set_halign: gtk::Align::Center,

            gtk::Image {
                set_icon_name: Some("dialog-error-symbolic"),
                set_pixel_size: 64,
                add_css_class: "error",
            },

            gtk::Label {
                set_markup: "<b><span size='large'>Application Crashed</span></b>",
            },

            gtk::Label {
                #[watch]
                set_label: &model.message,
                set_wrap: true,
                set_max_width_chars: 80,
                set_selectable: true,
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 10,
                set_halign: gtk::Align::Center,

                gtk::Button {
                    set_label: "Generate Debug Snapshot",
                    connect_clicked[sender] => move |_| {
                        sender.output(AppMsg::DebugSnapshot).expect("Channel closed");
                    }
                },

                gtk::Button {
                    set_label: "Exit",
                    connect_clicked => |_| {
                        std::process::exit(1);
                    }
                },
            }
        }
    }

    fn init(
        message: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self { message };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        self.message = msg;
    }
}
