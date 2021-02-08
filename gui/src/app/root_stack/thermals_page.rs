mod fan_curve_frame;

use daemon::gpu_controller::{FanControlInfo, GpuStats};
use gtk::prelude::*;
use gtk::*;

use fan_curve_frame::FanCurveFrame;

pub struct ThermalsSettings {
    pub automatic_fan_control_enabled: bool,
}

#[derive(Clone)]
pub struct ThermalsPage {
    pub container: Box,
    temp_label: Label,
    fan_speed_label: Label,
    fan_control_enabled_switch: Switch,
    fan_curve_frame: FanCurveFrame,
}

impl ThermalsPage {
    pub fn new() -> Self {
        let container = Box::new(Orientation::Vertical, 5);

        let grid = Grid::new();

        grid.set_margin_start(5);
        grid.set_margin_end(5);
        grid.set_margin_bottom(5);
        grid.set_margin_top(5);

        grid.set_column_homogeneous(true);

        grid.set_row_spacing(7);
        grid.set_column_spacing(5);

        grid.attach(
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

        grid.attach(&temp_label, 2, 0, 1, 1);

        grid.attach(
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

        grid.attach(&fan_speed_label, 2, 1, 1, 1);

        grid.attach(
            &{
                let label = Label::new(Some("Automatic fan control:"));
                label.set_halign(Align::End);
                label
            },
            0,
            2,
            1,
            1,
        );

        let fan_control_enabled_switch = Switch::new();

        fan_control_enabled_switch.set_active(true);
        fan_control_enabled_switch.set_halign(Align::Start);

        grid.attach(&fan_control_enabled_switch, 2, 2, 1, 1);

        container.pack_start(&grid, false, false, 5);

        let fan_curve_frame = FanCurveFrame::new();

        container.pack_start(&fan_curve_frame.container, true, true, 5);

        Self {
            container,
            temp_label,
            fan_speed_label,
            fan_control_enabled_switch,
            fan_curve_frame,
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

    pub fn set_ventilation_info(&self, fan_control_info: FanControlInfo) {
        self.fan_control_enabled_switch
            .set_active(!fan_control_info.enabled);

        if fan_control_info.enabled {
            self.fan_curve_frame.container.set_visible(true);
        } else {
            self.fan_curve_frame.container.set_visible(false);
        }
    }

    pub fn connect_settings_changed<F: Fn() + 'static>(&self, f: F) {
        self.fan_control_enabled_switch
            .connect_changed_active(move |_| {
                f();
            });
    }

    pub fn get_thermals_settings(&self) -> ThermalsSettings {
        let automatic_fan_control_enabled = self.fan_control_enabled_switch.get_active();

        ThermalsSettings {
            automatic_fan_control_enabled,
        }
    }
}
