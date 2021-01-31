use daemon::gpu_controller::GpuStats;
use gtk::*;

#[derive(Clone)]
pub struct ThermalsPage {
    pub container: Grid,
    temp_label: Label,
    fan_speed_label: Label,
}

impl ThermalsPage {
    pub fn new() -> Self {
        let container = Grid::new();

        container.set_margin_start(5);
        container.set_margin_end(5);
        container.set_margin_bottom(5);
        container.set_margin_top(5);

        container.set_column_homogeneous(true);

        container.set_row_spacing(7);
        container.set_column_spacing(5);

        container.attach(
            &{
                let label = Label::new(Some("Temperature:"));
                label.set_halign(Align::End);
                label
            },
            0,
            0,
            1,
            1,
        );

        let temp_label = Label::new(None);
        temp_label.set_halign(Align::Start);

        container.attach(&temp_label, 2, 0, 1, 1);

        container.attach(
            &{
                let label = Label::new(Some("Fan speed:"));
                label.set_halign(Align::End);
                label
            },
            0,
            1,
            1,
            1,
        );

        let fan_speed_label = Label::new(None);
        fan_speed_label.set_halign(Align::Start);

        container.attach(&fan_speed_label, 2, 1, 1, 1);

        Self {
            container,
            temp_label,
            fan_speed_label,
        }
    }

    pub fn set_thermals_info(&self, stats: &GpuStats) {
        self.temp_label
            .set_markup(&format!("<b>{}°C</b>", stats.gpu_temp));
        self.fan_speed_label.set_markup(&format!(
            "<b>{} RPM ({}%)</b>",
            stats.fan_speed,
            (stats.fan_speed as f64 / stats.max_fan_speed as f64 * 100.0).round()
        ));
    }
}
