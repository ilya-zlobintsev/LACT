use crate::I18N;
use gtk::prelude::*;
use i18n_embed_fl::fl;
use lact_schema::request::ProfileBase;
use relm4::{
    Component, ComponentController, ComponentParts, ComponentSender, Controller, RelmWidgetExt,
};
use relm4_components::simple_combo_box::SimpleComboBox;

pub struct NewProfileDialog {
    name_buffer: gtk::EntryBuffer,
    base_selector: Controller<SimpleComboBox<ProfileBase>>,
}

#[derive(Debug)]
pub enum NewProfileDialogMsg {
    Create,
}

#[relm4::component(pub)]
impl Component for NewProfileDialog {
    type Init = Vec<String>;
    type Input = NewProfileDialogMsg;
    type Output = (String, ProfileBase);
    type CommandOutput = ();

    view! {
        gtk::Window {
            set_default_size: (250, 130),
            set_title: Some(&fl!(I18N, "create-profile")),
            set_hide_on_close: true,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 5,
                set_margin_all: 10,

                gtk::Entry {
                    set_placeholder_text: Some(&fl!(I18N, "name")),
                    set_buffer: &model.name_buffer,
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 5,

                    gtk::Label {
                        set_label: "Copy settings from:",
                    },

                    #[local_ref]
                    base_selector -> gtk::ComboBoxText {
                        set_margin_horizontal: 5,
                        set_hexpand: true,
                        set_halign: gtk::Align::End,
                    },
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 5,
                    set_hexpand: true,
                    set_vexpand: true,
                    set_valign: gtk::Align::End,

                    gtk::Button {
                        set_label: &fl!(I18N, "cancel"),
                        set_hexpand: true,

                        connect_clicked[root] => move |_| {
                            root.hide();
                        },
                    },

                    gtk::Button {
                        set_label: &fl!(I18N, "create"),
                        set_hexpand: true,

                        connect_clicked => NewProfileDialogMsg::Create,
                    },
                }
            },
        }
    }

    fn init(
        current_profiles: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut variants = vec![ProfileBase::Empty, ProfileBase::Default];
        variants.extend(current_profiles.into_iter().map(ProfileBase::Profile));

        let base_selector = SimpleComboBox::<ProfileBase>::builder()
            .launch(SimpleComboBox {
                variants,
                active_index: Some(1),
            })
            .detach();

        let model = Self {
            base_selector,
            name_buffer: gtk::EntryBuffer::default(),
        };

        let base_selector = model.base_selector.widget();

        let widgets = view_output!();

        root.present();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, root: &Self::Root) {
        match msg {
            NewProfileDialogMsg::Create => {
                if self.name_buffer.length() != 0 {
                    if let Some(selected) = self.base_selector.model().active_index {
                        let base = self.base_selector.model().variants[selected].clone();
                        sender
                            .output((self.name_buffer.text().to_string(), base))
                            .unwrap();

                        root.hide();
                    }
                }
            }
        }
    }
}
