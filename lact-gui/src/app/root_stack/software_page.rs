use gtk::prelude::*;
use gtk::*;
use lact_client::schema::SystemInfo;

pub fn software_page(system_info: SystemInfo, embedded: bool) -> Grid {
    let container = Grid::new();

    container.set_margin_start(5);
    container.set_margin_end(5);
    container.set_margin_bottom(5);
    container.set_margin_top(5);

    container.set_column_spacing(5);

    container.attach(
        &{
            let label = Label::new(None);
            label.set_markup("LACT Daemon:");
            label.set_halign(Align::End);
            label.set_hexpand(true);
            label
        },
        0,
        0,
        1,
        1,
    );
    let mut daemon_version = format!("{}-{}", system_info.version, system_info.profile);
    if embedded {
        daemon_version.push_str("-embedded");
    }
    let daemon_version_label = Label::builder()
        .use_markup(true)
        .label(format!("<b>{daemon_version}</b>"))
        .hexpand(true)
        .halign(Align::Start)
        .build();

    container.attach(&daemon_version_label, 1, 0, 1, 1);

    container.attach(
        &{
            let label = Label::new(None);
            label.set_markup("LACT GUI:");
            label.set_halign(Align::End);
            label.set_hexpand(true);
            label
        },
        0,
        1,
        1,
        1,
    );

    let gui_version = env!("CARGO_PKG_VERSION");
    let gui_profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    let gui_version = format!("{gui_version}-{gui_profile}");

    let gui_version_label = Label::builder()
        .use_markup(true)
        .label(format!("<b>{gui_version}</b>"))
        .hexpand(true)
        .halign(Align::Start)
        .build();

    container.attach(&gui_version_label, 1, 1, 1, 1);

    container.attach(
        &Label::builder()
            .label("Kernel version:")
            .halign(Align::End)
            .hexpand(true)
            .build(),
        0,
        2,
        1,
        1,
    );
    let kernel_version_label = Label::builder()
        .use_markup(true)
        .label(format!("<b>{}</b>", system_info.kernel_version))
        .hexpand(true)
        .halign(Align::Start)
        .build();
    container.attach(&kernel_version_label, 1, 2, 1, 1);

    container
}
