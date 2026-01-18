use crate::I18N;
use gtk::prelude::{BoxExt, ButtonExt, OrientableExt, WidgetExt};
use i18n_embed_fl::fl;
use relm4::{css, ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent};

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

            gtk::ActionBar {
                pack_end = &gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 5,
                    set_margin_horizontal: 60,

                    gtk::Button {
                        set_label: &fl!(I18N, "revert-button"),
                        set_width_request: 150,
                        connect_clicked[sender] => move |_| {
                            sender.output(super::AppMsg::RevertChanges).unwrap();
                        },
                    },
                    gtk::Button {
                        set_label: &fl!(I18N, "apply-button"),
                        add_css_class: css::SUGGESTED_ACTION,
                        set_width_request: 150,
                        connect_clicked[sender] => move |_| {
                            sender.output(super::AppMsg::ApplyChanges).unwrap();
                        },
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
