use gtk::prelude::*;
use tracing::warn;

#[cfg(feature = "libadwaita")]
use libadwaita::prelude::MessageDialogExt;

#[cfg(feature = "libadwaita")]
#[macro_export]
macro_rules! info_dialog {
    ($parent:expr, $heading:expr, $body:expr, $response_id:expr, $response_txt:expr) => {{
        let diag = libadwaita::MessageDialog::builder()
            .heading($heading)
            .body($body)
            .modal(true)
            .transient_for($parent)
            .build();

        diag.add_response($response_id, $response_txt);

        diag.present();

        diag
    }};
}

#[cfg(not(feature = "libadwaita"))]
#[macro_export]
macro_rules! info_dialog {
    ($parent:expr, $heading:expr, $body:expr, $response_id:expr, $response_txt:expr) => {{
        let diag = gtk::MessageDialog::builder()
            .title($heading)
            .text($body)
            .modal(true)
            .transient_for($parent)
            .build();

        diag.add_button($response_txt, gtk::ResponseType::Close);

        diag.run_async(move |d, _| {
            d.close();
        });

        diag
    }};
}

pub fn show_error(parent: &impl IsA<gtk::Window>, err: anyhow::Error) {
    let text = format!("{err:?}");
    warn!("{}", text.trim());

    info_dialog!(parent, "Error", &text, "close", "_Close");
}
