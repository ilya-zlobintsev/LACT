use gtk::*;

#[derive(Debug, Clone)]
pub struct SoftwarePage {
    pub container: Grid,
    lact_version_label: Label,
}

impl SoftwarePage {
    pub fn new() -> Self {
        let container = Grid::new();

        container.set_margin_start(5);
        container.set_margin_end(5);
        container.set_margin_bottom(5);
        container.set_margin_top(5);

        container.set_column_spacing(10);

        container.attach(
            &{
                let label = Label::new(None);
                label.set_markup("<b>LACT Version:</b>");
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

        lact_version_label.set_markup(&format!("{}-{}", lact_version, lact_release_type));

        lact_version_label.set_hexpand(true);
        lact_version_label.set_halign(Align::Start);

        container.attach(&lact_version_label, 1, 0, 1, 1);

        Self {
            container,
            lact_version_label,
        }
    }
}
