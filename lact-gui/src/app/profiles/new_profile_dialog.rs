use crate::I18N;
use adw::prelude::{AdwDialogExt, EntryRowExt, PreferencesPageExt, PreferencesRowExt};
use gtk::prelude::*;
use i18n_embed_fl::fl;
use lact_schema::request::ProfileBase;
use relm4::{
    Component, ComponentController, ComponentParts, ComponentSender, Controller, RelmWidgetExt, css,
};
use relm4_components::simple_combo_box::SimpleComboBox;

pub struct NewProfileDialog {
    base_selector: Controller<SimpleComboBox<ProfileBase>>,
    parent: gtk::Widget,
}

#[derive(Debug)]
pub enum NewProfileDialogMsg {
    Create,
}

#[relm4::component(pub)]
impl Component for NewProfileDialog {
    type Init = (Vec<String>, gtk::Widget);
    type Input = NewProfileDialogMsg;
    type Output = (String, ProfileBase);
    type CommandOutput = ();

    view! {
        #[root]
        adw::Dialog {
            set_content_width: 420,
            set_title: &fl!(I18N, "create-profile"),

            #[wrap(Some)]
            set_child = &adw::ToolbarView {
                add_top_bar = &adw::HeaderBar {},

                #[wrap(Some)]
                set_content = &adw::PreferencesPage {
                    add = &adw::PreferencesGroup {
                        #[name = "name_entry"]
                        adw::EntryRow {
                            set_title: &fl!(I18N, "name"),
                            connect_entry_activated => NewProfileDialogMsg::Create,
                        },

                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 10,
                            set_margin_top: 10,

                            gtk::Label {
                                set_label: &fl!(I18N, "profile-copy-from"),
                                set_hexpand: true,
                                set_xalign: 0.0,
                            },

                            #[local_ref]
                            base_selector -> gtk::ComboBoxText {
                                set_valign: gtk::Align::Center,
                            },
                        },
                    },
                },

                add_bottom_bar = &gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 10,
                    set_margin_horizontal: 10,
                    set_margin_bottom: 10,

                    gtk::Button {
                        set_label: &fl!(I18N, "cancel"),
                        set_hexpand: true,

                        connect_clicked[root = root.downgrade()] => move |_| {
                            if let Some(root) = root.upgrade() {
                                root.close();
                            }
                        },
                    },

                    gtk::Button {
                        set_label: &fl!(I18N, "create"),
                        set_hexpand: true,
                        add_css_class: css::SUGGESTED_ACTION,

                        connect_clicked => NewProfileDialogMsg::Create,
                    },
                }
            },
        }
    }

    fn init(
        (current_profiles, parent): Self::Init,
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
            parent,
        };

        let base_selector = model.base_selector.widget();

        let widgets = view_output!();

        root.set_focus(Some(&widgets.name_entry));
        root.present(Some(&model.parent));

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
            NewProfileDialogMsg::Create => {
                if !widgets.name_entry.text().is_empty()
                    && let Some(selected) = self.base_selector.model().active_index
                {
                    let base = self.base_selector.model().variants[selected].clone();
                    sender
                        .output((widgets.name_entry.text().to_string(), base))
                        .unwrap();

                    root.close();
                }
            }
        }

        self.update_view(widgets, sender);
    }
}
