use gtk::prelude::{
    BoxExt, DialogExt, DialogExtManual, EditableExt, EntryExt, GtkWindowExt, OrientableExt,
    WidgetExt,
};
use relm4::{ComponentParts, ComponentSender, RelmWidgetExt};

pub struct ProfileRenameDialog {}

#[relm4::component(pub)]
impl relm4::SimpleComponent for ProfileRenameDialog {
    type Init = (String, gtk::Window);
    type Input = ();
    type Output = String;

    view! {
        gtk::Dialog {
            set_default_size: (400, 50),
            set_title: Some("Rename profile"),
            set_transient_for: Some(&root_window),
            set_hide_on_close: true,
            connect_response[root, sender, name_entry] => move |_, response| {
                match response {
                    gtk::ResponseType::Accept => {
                        sender.output(name_entry.text().to_string()).unwrap();
                        root.close();
                    }
                    gtk::ResponseType::Cancel => root.close(),
                    _ => (),
                }
            },
            add_buttons: &[("Cancel", gtk::ResponseType::Cancel), ("Save", gtk::ResponseType::Accept)],

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_margin_all: 5,
                set_spacing: 5,

                gtk::Label {
                    set_markup: &format!("Rename profile <b>{old_name}</b> to:"),
                },

                #[name = "name_entry"]
                gtk::Entry {
                    set_text: &old_name,
                    set_hexpand: true,
                    connect_activate[root] => move |_| {
                        root.response(gtk::ResponseType::Accept);
                    }
                },
            }
        },
    }

    fn init(
        (old_name, root_window): Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {};

        let widgets = view_output!();

        root.present();

        ComponentParts { widgets, model }
    }
}
