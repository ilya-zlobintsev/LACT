mod point_adjustment;

use self::point_adjustment::PointAdjustment;
use crate::app::pages::oc_adjustment::OcAdjustment;
use glib::{clone, Propagation};
use gtk::graphene::Point;
use gtk::gsk::Transform;
use gtk::prelude::*;
use gtk::*;
use lact_client::schema::{default_fan_curve, FanCurveMap};
use lact_schema::PmfwInfo;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

const DEFAULT_CHANGE_THRESHOLD: u64 = 2;
const DEFAULT_SPINDOWN_DELAY_MS: u64 = 5000;

#[derive(Clone)]
pub struct FanCurveFrame {
    pub container: Box,
    curve_container: Frame,
    zero_rpm_switch: Switch,
    zero_rpm_row: Box,
    points: Rc<RefCell<Vec<PointAdjustment>>>,
    spindown_delay_adj: OcAdjustment,
    change_threshold_adj: OcAdjustment,
    hysteresis_grid: Grid,
}

impl FanCurveFrame {
    pub fn new() -> Self {
        let root_box = Box::new(Orientation::Vertical, 5);

        let hbox = Box::new(Orientation::Horizontal, 5);

        let curve_container = Frame::new(Some("Fan Curve"));
        curve_container.set_hexpand(true);

        curve_container.set_margin_start(10);
        curve_container.set_margin_end(10);
        curve_container.set_margin_top(10);

        let ratio_title_label = Label::builder().label("Fan speed (%)").build();

        let fixed = Fixed::new();
        fixed.put(&ratio_title_label, 0.0, 0.0);

        // This is a workaround to rotate the label that only looks good at the default window size
        // Unfortunately there's no other way to do this (short of implementing custom rendering for a widget) as gtk4 removed the `angle` property for labels
        let rotation_transform = Transform::new()
            .rotate(-90.0)
            .translate(&Point::new(-200.0, 10.0));
        fixed.set_child_transform(&ratio_title_label, Some(&rotation_transform));

        hbox.append(&fixed);
        hbox.append(&curve_container);

        let temperature_title_label = Label::new(Some("Temperature (°C)"));

        let buttons_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(5)
            .halign(Align::End)
            .build();

        let add_button = Button::builder().icon_name("list-add-symbolic").build();
        let remove_button = Button::builder().icon_name("list-remove-symbolic").build();
        let default_button = Button::builder().label("Default").build();

        buttons_box.append(&default_button);
        buttons_box.append(&remove_button);
        buttons_box.append(&add_button);

        root_box.append(&hbox);
        root_box.append(&temperature_title_label);
        root_box.append(&buttons_box);

        let points = Rc::new(RefCell::new(Vec::new()));

        let hysteresis_grid = Grid::new();
        hysteresis_grid.set_margin_top(10);

        let spindown_delay_adj = oc_adjustment_row(
            &hysteresis_grid,
            0,
            "Spindown delay",
            "How long the GPU needs to remain at a lower temperature point for before ramping down the fan",
            " ms",
            OcAdjustmentOptions {
                default: DEFAULT_SPINDOWN_DELAY_MS as f64,
                min: 0.0,
                max: 30_000.0,
                step: 10.0,
                digits: 0,
            },
        );

        let change_threshold_adj = oc_adjustment_row(
            &hysteresis_grid,
            1,
            "Speed change threshold",
            "Hysteresis",
            "°C",
            OcAdjustmentOptions {
                default: DEFAULT_CHANGE_THRESHOLD as f64,
                min: 0.0,
                max: 10.0,
                step: 1.0,
                digits: 0,
            },
        );

        root_box.append(&hysteresis_grid);

        let zero_rpm_label = Label::builder()
            .label("Zero RPM mode")
            .halign(Align::Start)
            .build();
        let zero_rpm_switch = Switch::builder().halign(Align::End).hexpand(true).build();
        let zero_rpm_row = gtk::Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(5)
            .hexpand(true)
            .build();
        zero_rpm_row.append(&zero_rpm_label);
        zero_rpm_row.append(&zero_rpm_switch);

        root_box.append(&zero_rpm_row);

        let curve_frame = Self {
            container: root_box,
            curve_container,
            points,
            zero_rpm_row,
            zero_rpm_switch,
            spindown_delay_adj: spindown_delay_adj.clone(),
            change_threshold_adj: change_threshold_adj.clone(),
            hysteresis_grid,
        };

        default_button.connect_clicked(clone!(
            #[strong]
            curve_frame,
            move |_| {
                let curve = default_fan_curve();
                curve_frame.set_curve(&curve);
                spindown_delay_adj.set_value(DEFAULT_SPINDOWN_DELAY_MS as f64);
                change_threshold_adj.set_value(DEFAULT_CHANGE_THRESHOLD as f64);
            }
        ));

        add_button.connect_clicked(clone!(
            #[strong]
            curve_frame,
            move |_| {
                curve_frame.add_point();
            }
        ));

        remove_button.connect_clicked(clone!(
            #[strong]
            curve_frame,
            move |_| {
                curve_frame.remove_point();
            }
        ));

        curve_frame
    }

    fn add_point(&self) {
        let mut curve = self.get_curve();
        if let Some((temperature, ratio)) = curve.iter().last() {
            curve.insert(temperature + 5, *ratio);
            self.set_curve(&curve);
        } else {
            curve.insert(50, 0.5);
            self.set_curve(&curve);
        }
    }

    fn remove_point(&self) {
        let mut curve = self.get_curve();
        curve.pop_last();
        self.set_curve(&curve);
    }

    fn notify_changed(&self) {
        if let Some(point) = self.points.borrow().first() {
            point.ratio.emit_by_name::<()>("value-changed", &[]);
        }
    }

    pub fn set_curve(&self, curve: &FanCurveMap) {
        // Notify that the values were changed when the entire curve is overwritten, e.g. when resetting to default
        self.notify_changed();

        let points_container = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(5)
            .vexpand(true)
            .build();

        let mut adjustments = Vec::with_capacity(curve.len());

        for (temperature, ratio) in curve {
            let adjustment = PointAdjustment::new(&points_container, *ratio, *temperature);
            adjustments.push(adjustment);
        }

        self.points.replace(adjustments);
        self.curve_container.set_child(Some(&points_container));
    }

    pub fn get_curve(&self) -> FanCurveMap {
        let mut curve = BTreeMap::new();

        let points = self.points.borrow();
        for point in &*points {
            let temperature = point.temperature.value() as i32;
            let ratio = point.ratio.value() as f32;
            curve.insert(temperature, ratio);
        }

        curve
    }

    pub fn connect_adjusted<F: Fn() + 'static + Clone>(&self, f: F) {
        self.change_threshold_adj.connect_value_changed(clone!(
            #[strong]
            f,
            move |_| {
                f();
            }
        ));
        self.spindown_delay_adj.connect_value_changed(clone!(
            #[strong]
            f,
            move |_| {
                f();
            }
        ));

        let closure = clone!(
            #[strong]
            f,
            move |_: &Adjustment| {
                f();
            }
        );

        self.zero_rpm_switch.connect_state_set(clone!(
            #[strong]
            f,
            move |_, _| {
                f();
                Propagation::Proceed
            }
        ));

        for point in &*self.points.borrow() {
            point.ratio.connect_value_changed(closure.clone());
            point.temperature.connect_value_changed(closure.clone());
        }
    }

    pub fn set_change_threshold(&self, value: Option<u64>) {
        self.change_threshold_adj
            .set_initial_value(value.unwrap_or(0) as f64);
    }

    pub fn set_spindown_delay_ms(&self, value: Option<u64>) {
        self.spindown_delay_adj
            .set_initial_value(value.unwrap_or(0) as f64);
    }

    pub fn get_change_threshold(&self) -> u64 {
        self.change_threshold_adj.value() as u64
    }

    pub fn get_spindown_delay_ms(&self) -> u64 {
        self.spindown_delay_adj.value() as u64
    }

    pub fn set_hysteresis_settings_visibile(&self, visible: bool) {
        self.hysteresis_grid.set_visible(visible);
    }

    pub fn set_pmfw(&self, pmfw_info: &PmfwInfo) {
        self.zero_rpm_row
            .set_visible(pmfw_info.zero_rpm_enable.is_some());

        if let Some(value) = pmfw_info.zero_rpm_enable {
            self.zero_rpm_switch.set_active(value);
        }
    }

    pub fn get_zero_rpm(&self) -> Option<bool> {
        if self.zero_rpm_row.is_visible() {
            Some(self.zero_rpm_switch.is_active())
        } else {
            None
        }
    }
}

struct OcAdjustmentOptions {
    default: f64,
    min: f64,
    max: f64,
    step: f64,
    digits: i32,
}

fn oc_adjustment_row(
    grid: &Grid,
    row: i32,
    label: &str,
    tooltip: &str,
    unit: &'static str,
    opts: OcAdjustmentOptions,
) -> OcAdjustment {
    let label = Label::builder()
        .label(label)
        .halign(Align::Start)
        .tooltip_text(tooltip)
        .build();
    let adjustment = OcAdjustment::new(
        opts.default,
        opts.min,
        opts.max,
        opts.step,
        opts.step,
        opts.step,
    );

    let scale = Scale::builder()
        .orientation(Orientation::Horizontal)
        .adjustment(&adjustment)
        .hexpand(true)
        .round_digits(opts.digits)
        .digits(opts.digits)
        .value_pos(PositionType::Right)
        .margin_start(5)
        .margin_end(5)
        .build();

    let value_selector = SpinButton::new(Some(&adjustment), opts.step, opts.digits as u32);

    let value_label = Label::new(Some(&format!("{}{unit}", opts.default)));

    let popover = Popover::builder().child(&value_selector).build();
    let value_button = MenuButton::builder()
        .popover(&popover)
        .child(&value_label)
        .build();

    adjustment.connect_value_changed(clone!(
        #[strong]
        value_label,
        move |adjustment| {
            let value = match opts.digits {
                0 => adjustment.value().round(),
                _ => {
                    let rounding = opts.digits as f64 * 10.0;
                    (adjustment.value() * rounding).round() / rounding
                }
            };
            value_label.set_text(&format!("{value}{unit}"));
        }
    ));

    grid.attach(&label, 0, row, 1, 1);
    grid.attach(&scale, 1, row, 4, 1);
    grid.attach(&value_button, 6, row, 4, 1);

    adjustment
}

#[cfg(all(test, feature = "gtk-tests"))]
mod tests {
    use super::FanCurveFrame;
    use lact_client::schema::default_fan_curve;
    use pretty_assertions::assert_eq;

    #[test]
    fn set_get_curve() {
        gtk::init().unwrap();

        let curve = default_fan_curve();
        let frame = FanCurveFrame::new();
        frame.set_curve(&curve);
        let received_curve = frame.get_curve();
        assert_eq!(received_curve, curve);
    }
}
