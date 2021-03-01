use daemon::gpu_controller::GpuStats;
use gtk::*;

#[derive(Clone)]
pub struct StatsGrid {
    pub container: Grid,
    vram_usage_bar: LevelBar,
    vram_usage_label: Label,
    gpu_clock_label: Label,
    vram_clock_label: Label,
    gpu_voltage_label: Label,
    power_usage_label: Label,
}

impl StatsGrid {
    pub fn new() -> Self {
        let container = Grid::new();

        container.set_column_homogeneous(true);

        container.set_row_spacing(7);

        container.attach(&Label::new(Some("VRAM Usage")), 0, 0, 1, 1);

        let vram_usage_overlay = Overlay::new();

        let vram_usage_bar = LevelBar::new();

        let vram_usage_label = Label::new(None);

        {
            vram_usage_bar.set_orientation(Orientation::Horizontal);
            vram_usage_bar.set_value(1.0);

            vram_usage_label.set_text("0/0 MiB");

            vram_usage_overlay.add(&vram_usage_bar);
            vram_usage_overlay.add_overlay(&vram_usage_label);

            container.attach(&vram_usage_overlay, 1, 0, 1, 1);
        }

        let gpu_clock_label = Label::new(None);
        {
            let gpu_clock_box = Box::new(Orientation::Horizontal, 5);

            gpu_clock_box.pack_start(&Label::new(Some("GPU Clock:")), false, false, 2);

            gpu_clock_label.set_markup("<b>0MHz</b>");

            gpu_clock_box.pack_start(&gpu_clock_label, false, false, 2);

            gpu_clock_box.set_halign(Align::Center);

            container.attach(&gpu_clock_box, 0, 1, 1, 1);
        }

        let vram_clock_label = Label::new(None);
        {
            let vram_clock_box = Box::new(Orientation::Horizontal, 5);

            vram_clock_box.pack_start(&Label::new(Some("VRAM Clock:")), false, false, 2);

            vram_clock_label.set_markup("<b>0MHz</b>");

            vram_clock_box.pack_start(&vram_clock_label, false, false, 2);

            vram_clock_box.set_halign(Align::Center);

            container.attach(&vram_clock_box, 1, 1, 1, 1);
        }
        let gpu_voltage_label = Label::new(None);
        {
            let gpu_voltage_box = Box::new(Orientation::Horizontal, 5);

            gpu_voltage_box.pack_start(&Label::new(Some("GPU Voltage:")), false, false, 2);

            gpu_voltage_label.set_markup("<b>0.000V</b>");

            gpu_voltage_box.pack_start(&gpu_voltage_label, false, false, 2);

            gpu_voltage_box.set_halign(Align::Center);

            container.attach(&gpu_voltage_box, 0, 2, 1, 1);
        }

        let power_usage_label = Label::new(None);
        {
            let power_usage_box = Box::new(Orientation::Horizontal, 5);

            power_usage_box.pack_start(&Label::new(Some("Power Usage:")), false, false, 2);

            power_usage_label.set_markup("<b>00/000W</b>");

            power_usage_box.pack_start(&power_usage_label, false, false, 2);

            power_usage_box.set_halign(Align::Center);

            container.attach(&power_usage_box, 1, 2, 1, 1);
        }

        Self {
            container,
            vram_usage_bar,
            vram_usage_label,
            gpu_clock_label,
            vram_clock_label,
            gpu_voltage_label,
            power_usage_label,
        }
    }

    pub fn set_stats(&self, stats: &GpuStats) {
        self.vram_usage_bar.set_value(
            stats.mem_used.unwrap_or_else(|| 0) as f64
                / stats.mem_total.unwrap_or_else(|| 0) as f64,
        );
        self.vram_usage_label.set_text(&format!(
            "{}/{} MiB",
            stats.mem_used.unwrap_or_else(|| 0),
            stats.mem_total.unwrap_or_else(|| 0)
        ));

        self.gpu_clock_label.set_markup(&format!(
            "<b>{}MHz</b>",
            stats.gpu_freq.unwrap_or_else(|| 0)
        ));

        self.vram_clock_label.set_markup(&format!(
            "<b>{}MHz</b>",
            stats.mem_freq.unwrap_or_else(|| 0)
        ));

        self.gpu_voltage_label.set_markup(&format!(
            "<b>{}V</b>",
            stats.voltage.unwrap_or_else(|| 0) as f64 / 1000f64
        ));

        self.power_usage_label.set_markup(&format!(
            "<b>{}/{}W</b>",
            stats.power_avg.unwrap_or_else(|| 0),
            stats.power_cap.unwrap_or_else(|| 0)
        ));
    }
}
