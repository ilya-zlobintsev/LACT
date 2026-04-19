use crate::{APP_ID, GUI_VERSION, I18N, REPO_URL};
use adw::prelude::AdwDialogExt;
use i18n_embed_fl::fl;
use lact_schema::GIT_COMMIT;
use relm4::{ComponentParts, ComponentSender};

pub struct AboutDialog {
    parent: adw::ApplicationWindow,
}

#[derive(Debug)]
pub enum AboutDialogMsg {
    Show,
}

#[relm4::component(pub)]
impl relm4::Component for AboutDialog {
    type Init = adw::ApplicationWindow;
    type Input = AboutDialogMsg;
    type Output = ();
    type CommandOutput = ();

    view! {
        #[root]
        adw::AboutDialog {
            set_application_name: &fl!(I18N, "lact-gui"),
            set_application_icon: APP_ID,
            set_version: &format!("{GUI_VERSION} ({GIT_COMMIT})"),
            set_website: &format!("{REPO_URL}/wiki"),
            set_issue_url: &format!("{REPO_URL}/issues"),
            set_license_type: gtk::License::MitX11,
        }
    }

    fn init(
        parent: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = AboutDialog { parent };
        let widgets = view_output!();
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
            AboutDialogMsg::Show => {
                root.present(Some(&self.parent));
            }
        }
        self.update_view(widgets, sender);
    }
}
