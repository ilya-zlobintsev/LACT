mod fan_curve_frame;

use self::fan_curve_frame::FanCurveFrame;
use super::{list_clamp, LabelRow};
use crate::{app::page_section::PageSection, info_dialog};
use glib::clone;
use gtk::prelude::*;
use gtk::*;
use lact_client::schema::{default_fan_curve, DeviceStats, FanControlMode, FanCurveMap};
use libadwaita::prelude::*;

#[derive(Debug)]
pub struct ThermalsSettings {
    pub manual_fan_control: bool,
    pub mode: Option<FanControlMode>,
    pub static_speed: Option<f64>,
    pub curve: Option<FanCurveMap>,
}

#[derive(Debug, Clone)]
pub struct ThermalsPage {
    pub container: ScrolledWindow,
    temperatures_row: LabelRow,
    fan_speed_row: LabelRow,
    fan_static_speed_adjustment: Adjustment,
    fan_curve_frame: FanCurveFrame,
    fan_control_mode_stack: Stack,
    fan_control_mode_stack_switcher: StackSwitcher,
}

impl ThermalsPage {
    pub fn new(root_win: &impl IsA<Window>) -> Self {
        let vbox = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(12)
            .build();

        let stats_section = PageSection::new("Statistics");
        let stats_listbox = ListBox::builder()
            .css_classes(["boxed-list"])
            .selection_mode(SelectionMode::None)
            .build();

        let temperatures_row = LabelRow::new("Temperatures");

        let fan_speed_row = LabelRow::new("Fan speed");

        stats_listbox.append(&temperatures_row.container);
        stats_listbox.append(&fan_speed_row.container);

        stats_section.append(&stats_listbox);

        vbox.append(&stats_section);

        let fan_curve_frame = FanCurveFrame::new();

        let fan_static_speed_frame = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(12)
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();
        let fan_static_speed_adjustment = static_speed_adj(&fan_static_speed_frame);

        let fan_control_section = PageSection::new("Fan control");

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

        fan_control_mode_stack.add_titled(
            &libadwaita::Bin::builder()
                .css_classes(["card"])
                .valign(Align::Start)
                .child(&fan_static_speed_frame)
                .build(),
            Some("static"),
            "Static",
        );

        fan_control_section.append(&fan_control_mode_stack_switcher);
        fan_control_section.append(&fan_control_mode_stack);

        vbox.append(&fan_control_section);

        fan_control_mode_stack.connect_visible_child_name_notify(
            clone!(@strong root_win => move |stack| {
                if stack.visible_child_name() == Some("automatic".into()) {
                    show_fan_control_warning(&root_win)
                }
            }),
        );

        let container = ScrolledWindow::builder()
            .hscrollbar_policy(PolicyType::Never)
            .child(&list_clamp(&vbox))
            .build();

        Self {
            container,
            temperatures_row,
            fan_speed_row,
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
            temperatures.join(" | ")
        };

        self.temperatures_row.set_content(&temperatures_text);

        self.fan_speed_row
            .set_content(&match stats.fan.speed_current {
                Some(fan_speed_current) => format!(
                    "{} RPM ({}%)",
                    fan_speed_current,
                    (fan_speed_current as f64
                        / stats.fan.speed_max.unwrap_or(fan_speed_current) as f64
                        * 100.0)
                        .round()
                ),
                None => "No fan detected".into(),
            });

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

            if !stats.fan.control_enabled && self.fan_curve_frame.get_curve().is_empty() {
                self.fan_curve_frame.set_curve(&default_fan_curve());
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
    let label = Label::builder().label("Speed").halign(Align::Start).build();

    let adjustment = Adjustment::new(0.0, 0.0, 100.0, 0.1, 1.0, 0.0);

    let scale = Scale::builder()
        .orientation(Orientation::Horizontal)
        .adjustment(&adjustment)
        .hexpand(true)
        .build();

    let value_selector = SpinButton::new(Some(&adjustment), 1.0, 1);
    let value_label = Label::builder().margin_start(12).margin_end(12).build();

    let popover = Popover::builder().child(&value_selector).build();
    let value_button = MenuButton::builder()
        .css_classes(["circular"])
        .popover(&popover)
        .child(&value_label)
        .build();

    adjustment.connect_value_changed(clone!(@strong value_label => move |adjustment| {
        value_label.set_text(&format!("{:.1}%", adjustment.value()));
    }));

    adjustment.set_value(50.0);

    parent_box.append(&label);
    parent_box.append(&scale);
    parent_box.append(&value_button);

    adjustment
}

fn show_fan_control_warning(root_win: &impl IsA<Window>) {
    info_dialog!(
        root_win,
        "Warning",
        concat!(
            "Due to a driver bug, a reboot may be required for fan control ",
            "to properly switch back to automatic"
        ),
        "ok",
        "_Ok"
    );
}
