use gtk::prelude::{DialogExt, GtkWindowExt, WidgetExt};
use relm4::{ComponentParts, ComponentSender, SimpleComponent};

#[derive(Clone, Debug)]
pub struct ConfirmationOptions {
    pub title: String,
    pub message: String,
    pub buttons_type: gtk::ButtonsType,
}

pub struct ConfirmationDialog {}

#[relm4::component(pub)]
impl SimpleComponent for ConfirmationDialog {
    type Init = (ConfirmationOptions, gtk::ApplicationWindow);
    type Input = ();
    type Output = gtk::ResponseType;

    view! {
        gtk::MessageDialog {
            set_transient_for: Some(&parent),
            set_title: Some(&options.title),
            set_use_markup: true,

            connect_response[sender] => move |diag, response| {
                sender.output(response).unwrap();
                diag.close();
            },
        }
    }

    #[allow(unused_assignments)]
    fn init(
        (options, parent): Self::Init,
        mut root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {};

        root = gtk::MessageDialog::new(
            Some(&parent),
            gtk::DialogFlags::MODAL,
            gtk::MessageType::Question,
            options.buttons_type,
            options.message,
        );

        let widgets = view_output!();

        root.show();

        ComponentParts { model, widgets }
    }
}
