use crate::app::root_stack::{label_row, section_box};
use gtk::prelude::*;
use gtk::*;
use lact_client::schema::{ClockspeedStats, DeviceStats, PowerStats, VoltageStats, VramStats};

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
            .build();

        let gpu_clock_label = label_row("GPU Core Clock:", &labels_grid, 0, 0, false);
        let vram_clock_label = label_row("GPU Memory Clock:", &labels_grid, 0, 2, false);
        let gpu_voltage_label = label_row("GPU Voltage:", &labels_grid, 1, 0, false);
        let gpu_usage_label = label_row("GPU Usage:", &labels_grid, 1, 2, false);
        let gpu_temperature_label = label_row("GPU Temperature:", &labels_grid, 2, 0, false);
        let power_usage_label = label_row("GPU Power Usage:", &labels_grid, 2, 2, false);

        container.append(&labels_grid);

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
