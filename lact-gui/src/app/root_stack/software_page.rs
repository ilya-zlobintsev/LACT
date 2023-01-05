use gtk::prelude::*;
use gtk::*;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct SoftwarePage {
    pub container: Grid,
}

impl SoftwarePage {
    pub fn new(embedded_daemon: bool) -> Self {
        let container = Grid::new();

        container.set_margin_start(5);
        container.set_margin_end(5);
        container.set_margin_bottom(5);
        container.set_margin_top(5);

        container.set_column_spacing(5);

        container.attach(
            &{
                let label = Label::new(None);
                label.set_markup("LACT Version:");
                label.set_halign(Align::End);
                label.set_hexpand(true);
                label
            },
            0,
            0,
            1,
            1,
        );
        let lact_version_label = Label::new(None);

        let lact_version = env!("CARGO_PKG_VERSION");
        let lact_release_type = match cfg!(debug_assertions) {
            true => "debug",
            false => "release",
        };
        let mut version_text = format!("{lact_version}-{lact_release_type}");
        if embedded_daemon {
            version_text.push_str(" (embedded)");
        }

        lact_version_label.set_markup(&format!("<b>{version_text}</b>"));

        lact_version_label.set_hexpand(true);
        lact_version_label.set_halign(Align::Start);

        container.attach(&lact_version_label, 1, 0, 1, 1);

        container.attach(
            &Label::builder()
                .label("Kernel version:")
                .halign(Align::End)
                .hexpand(true)
                .build(),
            0,
            1,
            1,
            1,
        );
        let kernel_version_label = Label::builder()
            .use_markup(true)
            .label(&format!("<b>{}</b>", get_kernel_version().trim()))
            .hexpand(true)
            .halign(Align::Start)
            .build();
        container.attach(&kernel_version_label, 1, 1, 1, 1);

        Self { container }
    }
}

fn get_kernel_version() -> String {
    let output = Command::new("uname")
        .arg("-r")
        .output()
        .expect("Could not run uname");
    String::from_utf8(output.stdout).expect("Invalid uname output")
}
