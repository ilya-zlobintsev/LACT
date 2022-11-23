// mod fan_curve_frame;

use glib::clone;
use gtk::prelude::*;
use gtk::*;
use lact_schema::DeviceStats;
use std::collections::BTreeMap;
use tracing::trace;

// use fan_curve_frame::FanCurveFrame;

pub struct ThermalsSettings {
    pub automatic_fan_control_enabled: bool,
    pub curve: BTreeMap<i64, f64>,
}

#[derive(Clone)]
pub struct ThermalsPage {
    pub container: Box,
    temp_label: Label,
    fan_speed_label: Label,
    fan_control_enabled_switch: Switch,
    // fan_curve_frame: FanCurveFrame,
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
                let label = Label::new(Some("Temperatures:"));
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

        /*let fan_curve_frame = FanCurveFrame::new();

        container.pack_start(&fan_curve_frame.container, true, true, 5);

        // Show/hide fan curve when the switch is toggled
        {
            let fan_curve_frame = fan_curve_frame.clone();
            fan_control_enabled_switch.connect_changed_active(move |switch| {
                trace!("Fan control switch toggled");
                if switch.state() {
                    {
                        glib::idle_add(|| {
                            let diag = MessageDialog::new(None::<&Window>, DialogFlags::empty(), MessageType::Warning, ButtonsType::Ok,
                            "Warning! Due to a driver bug, a reboot may be required for fan control to properly switch back to automatic.");
                            diag.run();
                            diag.hide();
                            glib::Continue(false)
                        });
                    }

                    fan_curve_frame.hide();
                } else {
                    fan_curve_frame.show();
                }
            });
        }*/

        Self {
            container,
            temp_label,
            fan_speed_label,
            fan_control_enabled_switch,
            // fan_curve_frame,
        }
    }

    pub fn set_stats(&self, stats: &DeviceStats, initial: bool) {
        let mut temperatures: Vec<String> = stats
            .temps
            .iter()
            .filter_map(|(label, temp)| temp.current.map(|current| format!("{label}: {current}°C")))
            .collect();
        temperatures.sort();
        let temperatures_text = if temperatures.is_empty() {
            String::from("No sensors found")
        } else {
            temperatures.join("\n")
        };

        self.temp_label
            .set_markup(&format!("<b>{temperatures_text}</b>",));

        match stats.fan.speed_current {
            Some(fan_speed_current) => self.fan_speed_label.set_markup(&format!(
                "<b>{} RPM ({}%)</b>",
                fan_speed_current,
                (fan_speed_current as f64
                    / stats.fan.speed_max.unwrap_or(fan_speed_current) as f64
                    * 100.0)
                    .round()
            )),
            None => self.fan_speed_label.set_text("No fan detected"),
        }

        if initial {
            self.fan_control_enabled_switch.set_visible(true);
            self.fan_control_enabled_switch
                .set_active(!stats.fan.control_enabled);

            /*if stats.fan.control_enabled {
                self.fan_curve_frame.show();
            } else {
                self.fan_curve_frame.hide();
            }*/
        }

        // TODO
        // self.fan_curve_frame.set_curve(&fan_control_info.curve);
    }

    pub fn connect_settings_changed<F: Fn() + 'static + Clone>(&self, f: F) {
        self.fan_control_enabled_switch
            .connect_changed_active(clone!(@strong f => move |_| {
                f();
            }));

        /*self.fan_curve_frame.connect_adjusted(move || {
            f();
        });*/
    }

    /*pub fn get_thermals_settings(&self) -> ThermalsSettings {
        let automatic_fan_control_enabled = self.fan_control_enabled_switch.state();
        let curve = self.fan_curve_frame.get_curve();

        ThermalsSettings {
            automatic_fan_control_enabled,
            curve,
        }
    }*/

    pub fn hide_fan_controls(&self) {
        self.fan_control_enabled_switch.set_visible(false);
        // self.fan_curve_frame.hide();
    }
}
