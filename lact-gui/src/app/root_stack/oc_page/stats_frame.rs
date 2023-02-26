use gtk::prelude::*;
use gtk::*;
use lact_client::schema::{ClockspeedStats, DeviceStats, PowerStats, VoltageStats, VramStats};

use super::section_box;

#[derive(Clone)]
pub struct StatsFrame {
    pub container: Box,
    vram_usage_bar: LevelBar,
    vram_usage_label: Label,
    gpu_clock_label: Label,
    vram_clock_label: Label,
    gpu_voltage_label: Label,
    power_usage_label: Label,
    gpu_temperature_label: Label,
    gpu_usage_label: Label,
}

impl StatsFrame {
    pub fn new() -> Self {
        let container = section_box("Statistics");

        let vram_usage_hbox = Box::new(Orientation::Horizontal, 5);

        let vram_usage_title = name_label("VRAM Usage:");
        vram_usage_hbox.append(&vram_usage_title);

        let vram_usage_overlay = Overlay::new();
        let vram_usage_bar = LevelBar::builder()
            .hexpand(true)
            .value(1.0)
            .orientation(Orientation::Horizontal)
            .build();
        let vram_usage_label = Label::new(None);

        vram_usage_overlay.set_child(Some(&vram_usage_bar));
        vram_usage_overlay.add_overlay(&vram_usage_label);

        vram_usage_hbox.append(&vram_usage_overlay);

        container.append(&vram_usage_hbox);

        let labels_grid = Grid::builder()
            .row_spacing(5)
            .column_spacing(20)
            .margin_top(5)
            .margin_bottom(5)
            .build();

        let gpu_clock_label = label_row("GPU Core Clock:", &labels_grid, 0, 0);
        let vram_clock_label = label_row("GPU Memory Clock:", &labels_grid, 0, 2);
        let gpu_voltage_label = label_row("GPU Voltage:", &labels_grid, 1, 0);
        let gpu_usage_label = label_row("GPU Usage:", &labels_grid, 1, 2);
        let gpu_temperature_label = label_row("GPU Temperature:", &labels_grid, 2, 0);
        let power_usage_label = label_row("GPU Power Usage:", &labels_grid, 2, 2);

        container.append(&labels_grid);

        /*let gpu_clock_label = Label::new(None);
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
        }*/

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

        if let (Some(used_vram), Some(total_vram)) = (used_vram, total_vram) {
            self.vram_usage_bar
                .set_value(used_vram as f64 / total_vram as f64);
        }
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

fn name_label(text: &str) -> Label {
    Label::builder().label(text).halign(Align::Start).build()
}

fn label_row(title: &str, parent: &Grid, row: i32, column_offset: i32) -> Label {
    let title_label = Label::builder().label(title).halign(Align::Start).build();
    let value_label = Label::builder().halign(Align::Start).hexpand(true).build();

    parent.attach(&title_label, column_offset, row, 1, 1);
    parent.attach(&value_label, column_offset + 1, row, 1, 1);

    value_label
}
