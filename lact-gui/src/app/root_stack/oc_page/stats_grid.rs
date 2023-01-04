use gtk::prelude::*;
use gtk::*;
use lact_client::schema::{ClockspeedStats, DeviceStats, PowerStats, VoltageStats, VramStats};

#[derive(Clone)]
pub struct StatsGrid {
    pub container: Grid,
    vram_usage_bar: LevelBar,
    vram_usage_label: Label,
    gpu_clock_label: Label,
    vram_clock_label: Label,
    gpu_voltage_label: Label,
    power_usage_label: Label,
    gpu_temperature_label: Label,
    gpu_usage_label: Label,
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

            vram_usage_overlay.set_child(Some(&vram_usage_bar));
            vram_usage_overlay.add_overlay(&vram_usage_label);

            container.attach(&vram_usage_overlay, 1, 0, 2, 1);
        }

        let gpu_clock_label = Label::new(None);
        {
            let gpu_clock_box = Box::new(Orientation::Horizontal, 5);

            gpu_clock_box.append(&Label::new(Some("GPU Clock:")));

            gpu_clock_label.set_markup("<b>0MHz</b>");

            gpu_clock_box.append(&gpu_clock_label);

            gpu_clock_box.set_halign(Align::Center);

            container.attach(&gpu_clock_box, 0, 1, 1, 1);
        }

        let vram_clock_label = Label::new(None);
        {
            let vram_clock_box = Box::new(Orientation::Horizontal, 5);

            vram_clock_box.append(&Label::new(Some("VRAM Clock:")));

            vram_clock_label.set_markup("<b>0MHz</b>");

            vram_clock_box.append(&vram_clock_label);

            vram_clock_box.set_halign(Align::Center);

            container.attach(&vram_clock_box, 1, 1, 1, 1);
        }
        let gpu_voltage_label = Label::new(None);
        {
            let gpu_voltage_box = Box::new(Orientation::Horizontal, 5);

            gpu_voltage_box.append(&Label::new(Some("GPU Voltage:")));

            gpu_voltage_label.set_markup("<b>0.000V</b>");

            gpu_voltage_box.append(&gpu_voltage_label);

            gpu_voltage_box.set_halign(Align::Center);

            container.attach(&gpu_voltage_box, 2, 1, 1, 1);
        }

        let power_usage_label = Label::new(None);
        {
            let power_usage_box = Box::new(Orientation::Horizontal, 5);

            power_usage_box.append(&Label::new(Some("Power Usage:")));

            power_usage_label.set_markup("<b>00/000W</b>");

            power_usage_box.append(&power_usage_label);

            power_usage_box.set_halign(Align::Center);

            container.attach(&power_usage_box, 0, 2, 1, 1);
        }

        let gpu_temperature_label = Label::new(None);
        {
            let gpu_temperature_box = Box::new(Orientation::Horizontal, 5);

            gpu_temperature_box.append(&Label::new(Some("GPU Temperature:")));

            // gpu_temperature_label.set_markup("<b>0°C</b>");

            gpu_temperature_box.append(&gpu_temperature_label);

            gpu_temperature_box.set_halign(Align::Center);

            container.attach(&gpu_temperature_box, 1, 2, 1, 1);
        }

        let gpu_usage_label = Label::new(None);
        {
            let gpu_usage_box = Box::new(Orientation::Horizontal, 5);

            gpu_usage_box.append(&Label::new(Some("GPU Usage:")));

            gpu_usage_box.append(&gpu_usage_label);

            gpu_usage_box.set_halign(Align::Center);

            container.attach(&gpu_usage_box, 2, 2, 1, 1);
        }

        Self {
            container,
            vram_usage_bar,
            vram_usage_label,
            gpu_clock_label,
            vram_clock_label,
            gpu_voltage_label,
            power_usage_label,
            gpu_temperature_label,
            gpu_usage_label,
        }
    }

    pub fn set_stats(&self, stats: &DeviceStats) {
        let VramStats {
            total: total_vram,
            used: used_vram,
        } = stats.vram;

        self.vram_usage_bar
            .set_value(used_vram.unwrap_or(0) as f64 / total_vram.unwrap_or(0) as f64);
        self.vram_usage_label.set_text(&format!(
            "{}/{} MiB",
            used_vram.unwrap_or(0) / 1024 / 1024,
            total_vram.unwrap_or(0) / 1024 / 1024,
        ));

        let ClockspeedStats {
            gpu_clockspeed,
            vram_clockspeed,
        } = stats.clockspeed;

        self.gpu_clock_label
            .set_markup(&format!("<b>{}MHz</b>", gpu_clockspeed.unwrap_or(0)));
        self.vram_clock_label
            .set_markup(&format!("<b>{}MHz</b>", vram_clockspeed.unwrap_or(0)));

        let VoltageStats {
            gpu: gpu_voltage, ..
        } = stats.voltage;

        self.gpu_voltage_label.set_markup(&format!(
            "<b>{}V</b>",
            gpu_voltage.unwrap_or(0) as f64 / 1000f64
        ));

        let PowerStats {
            average: power_average,
            cap_current: power_cap_current,
            ..
        } = stats.power;

        self.power_usage_label.set_markup(&format!(
            "<b>{}/{}W</b>",
            power_average.unwrap_or(0.0),
            power_cap_current.unwrap_or(0.0)
        ));

        let maybe_temp = stats
            .temps
            .get("junction")
            .or_else(|| stats.temps.get("edge"));

        if let Some(temp) = maybe_temp.and_then(|temp| temp.current) {
            self.gpu_temperature_label
                .set_markup(&format!("<b>{temp}°C</b>"));
        }

        self.gpu_usage_label.set_markup(&format!(
            "<b>{}%</b>",
            stats.busy_percent.unwrap_or_default()
        ));
    }
}
