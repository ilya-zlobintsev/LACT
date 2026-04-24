use crate::I18N;
use adw::prelude::{AdwDialogExt, EntryRowExt, PreferencesPageExt, PreferencesRowExt};
use gtk::prelude::{BoxExt, ButtonExt, EditableExt, ObjectExt, OrientableExt, WidgetExt};
use i18n_embed_fl::fl;
use relm4::{css, ComponentParts, ComponentSender, RelmWidgetExt};

pub struct ProfileRenameDialog {
    old_name: String,
    parent: gtk::Widget,
}

#[derive(Debug)]
pub enum ProfileRenameDialogMsg {
    Save,
}

#[relm4::component(pub)]
impl relm4::Component for ProfileRenameDialog {
    type Init = (String, gtk::Widget);
    type Input = ProfileRenameDialogMsg;
    type Output = String;
    type CommandOutput = ();

    view! {
        #[root]
        adw::Dialog {
            set_content_width: 420,
            set_title: &fl!(I18N, "rename-profile"),

            #[wrap(Some)]
            set_child = &adw::ToolbarView {
                add_top_bar = &adw::HeaderBar {},

                #[wrap(Some)]
                set_content = &adw::PreferencesPage {
                    add = &adw::PreferencesGroup {
                        #[name = "name_entry"]
                        adw::EntryRow {
                            set_title: &fl!(I18N, "name"),
                            set_text: &model.old_name,
                            connect_entry_activated => ProfileRenameDialogMsg::Save,
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
                        set_label: &fl!(I18N, "save"),
                        set_hexpand: true,
                        add_css_class: css::SUGGESTED_ACTION,
                        connect_clicked => ProfileRenameDialogMsg::Save,
                    },
                },
            }
        },
    }

    fn init(
        (old_name, parent): Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self { old_name, parent };

        let widgets = view_output!();

        root.set_focus(Some(&widgets.name_entry));
        root.present(Some(&model.parent));

        ComponentParts { widgets, model }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: Self::Input,
        sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match msg {
            ProfileRenameDialogMsg::Save => {
                sender
                    .output(widgets.name_entry.text().to_string())
                    .unwrap();
                root.close();
            }
        }

        self.update_view(widgets, sender);
    }
}
