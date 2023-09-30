mod fan_curve_frame;

use glib::clone;
use gtk::prelude::*;
use gtk::*;
use lact_client::schema::{default_fan_curve, DeviceStats, FanControlMode, FanCurveMap};

use self::fan_curve_frame::FanCurveFrame;

use super::{label_row, section_box, values_grid};

#[derive(Debug)]
pub struct ThermalsSettings {
    pub manual_fan_control: bool,
    pub mode: Option<FanControlMode>,
    pub static_speed: Option<f64>,
    pub curve: Option<FanCurveMap>,
}

#[derive(Clone)]
pub struct ThermalsPage {
    pub container: Box,
    temperatures_label: Label,
    fan_speed_label: Label,
    fan_static_speed_adjustment: Adjustment,
    fan_curve_frame: FanCurveFrame,
    fan_control_mode_stack: Stack,
    fan_control_mode_stack_switcher: StackSwitcher,
}

impl ThermalsPage {
    pub fn new() -> Self {
        let container = Box::new(Orientation::Vertical, 15);

        let stats_section = section_box("Statistics");
        let stats_grid = values_grid();

        let temperatures_label = label_row("Temperatures:", &stats_grid, 0, 0, false);
        let fan_speed_label = label_row("Fan speed:", &stats_grid, 1, 0, false);

        stats_section.append(&stats_grid);

        container.append(&stats_section);

        let fan_curve_frame = FanCurveFrame::new();

        let fan_static_speed_frame = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(5)
            .valign(Align::Start)
            .build();
        let fan_static_speed_adjustment = static_speed_adj(&fan_static_speed_frame);

        let fan_control_section = section_box("Fan control");

        let fan_control_mode_stack = Stack::builder().build();
        let fan_control_mode_stack_switcher = StackSwitcher::builder()
            .stack(&fan_control_mode_stack)
            .visible(false)
            .sensitive(false)
            .build();

        fan_control_mode_stack.add_titled(
            &Box::new(Orientation::Vertical, 15),
            Some("automatic"),
            "Automatic",
        );

        fan_control_mode_stack.add_titled(&fan_curve_frame.container, Some("curve"), "Curve");

        fan_control_mode_stack.add_titled(&fan_static_speed_frame, Some("static"), "Static");

        fan_control_section.append(&fan_control_mode_stack_switcher);
        fan_control_section.append(&fan_control_mode_stack);

        container.append(&fan_control_section);

        fan_control_mode_stack.connect_visible_child_name_notify(|stack| {
            if stack.visible_child_name() == Some("automatic".into()) {
                show_fan_control_warning()
            }
        });

        Self {
            container,
            temperatures_label,
            fan_speed_label,
            fan_static_speed_adjustment,
            fan_curve_frame,
            fan_control_mode_stack,
            fan_control_mode_stack_switcher,
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
            temperatures.join(", ")
        };

        self.temperatures_label
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
            self.fan_control_mode_stack_switcher.set_visible(true);
            self.fan_control_mode_stack_switcher
                .set_sensitive(stats.fan.speed_current.is_some());

            let child_name = match stats.fan.control_mode {
                Some(mode) if stats.fan.control_enabled => match mode {
                    FanControlMode::Static => "static",
                    FanControlMode::Curve => "curve",
                },
                _ => "automatic",
            };

            self.fan_control_mode_stack
                .set_visible_child_name(child_name);

            if let Some(static_speed) = &stats.fan.static_speed {
                self.fan_static_speed_adjustment
                    .set_value(*static_speed * 100.0);
            }

            if let Some(curve) = &stats.fan.curve {
                self.fan_curve_frame.set_curve(curve);
            }

            if !stats.fan.control_enabled {
                if self.fan_curve_frame.get_curve().is_empty() {
                    self.fan_curve_frame.set_curve(&default_fan_curve());
                }
            }
        }
    }

    pub fn connect_settings_changed<F: Fn() + 'static + Clone>(&self, f: F) {
        self.fan_control_mode_stack
            .connect_visible_child_name_notify(clone!(@strong f => move |_| {
                f();
            }));

        self.fan_static_speed_adjustment
            .connect_value_changed(clone!(@strong f => move |_| {
                f();
            }));

        self.fan_curve_frame.connect_adjusted(move || {
            f();
        });
    }

    pub fn get_thermals_settings(&self) -> Option<ThermalsSettings> {
        if self.fan_control_mode_stack_switcher.is_sensitive() {
            let name = self.fan_control_mode_stack.visible_child_name();
            let name = name
                .as_ref()
                .map(|name| name.as_str())
                .expect("No name on the visible child");
            let (manual_fan_control, mode) = match name {
                "automatic" => (false, None),
                "curve" => (true, Some(FanControlMode::Curve)),
                "static" => (true, Some(FanControlMode::Static)),
                _ => unreachable!(),
            };
            let static_speed = Some(self.fan_static_speed_adjustment.value() / 100.0);
            let curve = self.fan_curve_frame.get_curve();
            let curve = if curve.is_empty() { None } else { Some(curve) };

            Some(ThermalsSettings {
                manual_fan_control,
                mode,
                static_speed,
                curve,
            })
        } else {
            None
        }
    }
}

fn static_speed_adj(parent_box: &Box) -> Adjustment {
    let label = Label::builder()
        .label("Speed (in %)")
        .halign(Align::Start)
        .build();

    let adjustment = Adjustment::new(0.0, 0.0, 100.0, 0.1, 1.0, 0.0);

    let scale = Scale::builder()
        .orientation(Orientation::Horizontal)
        .adjustment(&adjustment)
        .hexpand(true)
        // .draw_value(true)
        // .value_pos(PositionType::Left)
        .margin_start(5)
        .margin_end(5)
        .build();

    let value_selector = SpinButton::new(Some(&adjustment), 1.0, 1);
    let value_label = Label::new(None);

    let popover = Popover::builder().child(&value_selector).build();
    let value_button = MenuButton::builder()
        .popover(&popover)
        .child(&value_label)
        .build();

    adjustment.connect_value_changed(clone!(@strong value_label => move |adjustment| {
        let value = adjustment.value();
        value_label.set_text(&format!("{value:.1}"));
    }));

    adjustment.set_value(50.0);

    parent_box.append(&label);
    parent_box.append(&scale);
    parent_box.append(&value_button);

    adjustment
}

fn show_fan_control_warning() {
    let diag = MessageDialog::new(None::<&Window>, DialogFlags::empty(), MessageType::Warning, ButtonsType::Ok,
                        "Warning! Due to a driver bug, a reboot may be required for fan control to properly switch back to automatic.");
    diag.run_async(|diag, _| {
        diag.hide();
    })
}
