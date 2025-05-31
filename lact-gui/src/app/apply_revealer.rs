use gtk::prelude::{BoxExt, ButtonExt, OrientableExt, WidgetExt};
use relm4::{ComponentParts, ComponentSender, SimpleComponent};

pub struct ApplyRevealer {
    shown: bool,
}

#[derive(Debug)]
pub enum ApplyRevealerMsg {
    Show,
    Hide,
}

#[relm4::component(pub)]
impl SimpleComponent for ApplyRevealer {
    type Init = ();

    type Input = ApplyRevealerMsg;
    type Output = super::AppMsg;

    view! {
        gtk::Revealer {
            #[watch]
            set_reveal_child: model.shown,

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 5,

                gtk::Button {
                    set_label: "Apply",
                    set_hexpand: true,
                    connect_clicked[sender] => move |_| {
                        sender.output(super::AppMsg::ApplyChanges).unwrap();
                    },
                },

                gtk::Button {
                    set_label: "Revert",
                    connect_clicked[sender] => move |_| {
                        sender.output(super::AppMsg::RevertChanges).unwrap();
                    },
                },
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self { shown: false };

        let widgets = view_output!();

        ComponentParts { widgets, model }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            ApplyRevealerMsg::Show => self.shown = true,
            ApplyRevealerMsg::Hide => self.shown = false,
        }
    }
}
