use super::{list_clamp, LabelRow};
use crate::GUI_VERSION;
use lact_client::schema::SystemInfo;

pub fn software_page(system_info: SystemInfo, embedded: bool) -> gtk::ScrolledWindow {
    let listbox = gtk::ListBox::builder()
        .css_classes(["boxed-list"])
        .selection_mode(gtk::SelectionMode::None)
        .build();

    listbox.append(
        &LabelRow::new_with_content(
            "LACT Daemon",
            &format!(
                "{}-{}{}",
                system_info.version,
                system_info.profile,
                if embedded { "-embedded" } else { "" }
            ),
        )
        .container,
    );

    listbox.append(
        &LabelRow::new_with_content(
            "LACT GUI",
            &format!(
                "{}-{}",
                GUI_VERSION,
                if cfg!(debug_assertions) {
                    "debug"
                } else {
                    "release"
                }
            ),
        )
        .container,
    );

    listbox.append(&LabelRow::new_with_content("LACT GUI", &system_info.kernel_version).container);

    gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .child(&list_clamp(&listbox))
        .build()
}
