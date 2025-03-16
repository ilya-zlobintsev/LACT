use crate::app::pages::oc_adjustment::OcAdjustment;
use amdgpu_sysfs::gpu_handle::fan_control::FanInfo;
use gtk::{
    glib::{clone, Propagation},
    prelude::{AdjustmentExt, ButtonExt, GridExt, WidgetExt},
    Align, Button, Grid, Label, MenuButton, Orientation, Popover, Scale, SpinButton, Switch,
};
use lact_client::schema::{PmfwInfo, PmfwOptions};

#[derive(Clone)]
pub struct PmfwFrame {
    pub container: Grid,
    target_temperature: OcAdjustment,
    acoustic_limit: OcAdjustment,
    acoustic_target: OcAdjustment,
    minimum_pwm: OcAdjustment,
    zero_rpm_label: Label,
    zero_rpm_switch: Switch,
    pub zero_rpm_temperature: OcAdjustment,
    reset_button: Button,
}

impl PmfwFrame {
    pub fn new() -> Self {
        let grid = Grid::builder()
            .orientation(Orientation::Vertical)
            .row_spacing(5)
            .margin_top(10)
            .margin_bottom(10)
            .margin_start(10)
            .margin_end(10)
            .build();

        let target_temperature = adjustment(&grid, "Target temperature (°C)", 0);
        let acoustic_limit = adjustment(&grid, "Acoustic limit (RPM)", 1);
        let acoustic_target = adjustment(&grid, "Acoustic target (RPM)", 2);
        let minimum_pwm = adjustment(&grid, "Minimum fan speed (%)", 3);

        let zero_rpm_label = Label::builder()
            .label("Zero RPM mode")
            .halign(Align::Start)
            .build();
        let zero_rpm_switch = Switch::builder().halign(Align::End).build();

        grid.attach(&zero_rpm_label, 0, 4, 1, 1);
        grid.attach(&zero_rpm_switch, 5, 4, 1, 1);

        let zero_rpm_temperature = adjustment(&grid, "Zero RPM stop temperature (°C)", 5);

        let reset_button = Button::builder()
            .label("Reset")
            .halign(Align::Fill)
            .margin_top(5)
            .margin_bottom(5)
            .tooltip_text("Warning: this resets the fan firmware settings!")
            .css_classes(["destructive-action"])
            .visible(false)
            .build();
        grid.attach(&reset_button, 5, 6, 1, 1);

        Self {
            container: grid,
            target_temperature,
            acoustic_limit,
            acoustic_target,
            minimum_pwm,
            zero_rpm_temperature,
            zero_rpm_switch,
            zero_rpm_label,
            reset_button,
        }
    }

    pub fn set_info(&self, info: &PmfwInfo) {
        set_fan_info(&self.acoustic_limit, info.acoustic_limit);
        set_fan_info(&self.acoustic_target, info.acoustic_target);
        set_fan_info(&self.minimum_pwm, info.minimum_pwm);
        set_fan_info(&self.target_temperature, info.target_temp);
        set_fan_info(&self.zero_rpm_temperature, info.zero_rpm_temperature);

        if let Some(zero_rpm) = info.zero_rpm_enable {
            self.zero_rpm_switch.set_active(zero_rpm);

            self.zero_rpm_switch.set_visible(true);
            self.zero_rpm_label.set_visible(true);
        } else {
            self.zero_rpm_switch.set_visible(false);
            self.zero_rpm_label.set_visible(false);
        }

        let settings_available = *info != PmfwInfo::default();

        self.reset_button.set_visible(settings_available);
    }

    pub fn connect_settings_changed<F: Fn() + 'static + Clone>(&self, f: F) {
        self.acoustic_limit.connect_value_changed(clone!(
            #[strong]
            f,
            move |_| {
                f();
            }
        ));
        self.acoustic_target.connect_value_changed(clone!(
            #[strong]
            f,
            move |_| {
                f();
            }
        ));
        self.minimum_pwm.connect_value_changed(clone!(
            #[strong]
            f,
            move |_| {
                f();
            }
        ));
        self.target_temperature.connect_value_changed(clone!(
            #[strong]
            f,
            move |_| {
                f();
            }
        ));
        self.zero_rpm_temperature.connect_value_changed(clone!(
            #[strong]
            f,
            move |_| {
                f();
            }
        ));

        self.zero_rpm_switch.connect_state_set(clone!(
            #[strong]
            f,
            move |_, _| {
                f();
                Propagation::Proceed
            }
        ));
    }

    pub fn connect_reset<F: Fn() + 'static + Clone>(&self, f: F) {
        self.reset_button.connect_clicked(move |_| {
            f();
        });
    }

    pub fn get_pmfw_options(&self) -> PmfwOptions {
        PmfwOptions {
            acoustic_limit: self
                .acoustic_limit
                .get_nonzero_value()
                .map(|value| value as u32),
            acoustic_target: self
                .acoustic_target
                .get_nonzero_value()
                .map(|value| value as u32),
            minimum_pwm: self
                .minimum_pwm
                .get_nonzero_value()
                .map(|value| value as u32),
            target_temperature: self
                .target_temperature
                .get_nonzero_value()
                .map(|value| value as u32),
            zero_rpm_threshold: self
                .zero_rpm_temperature
                .get_nonzero_value()
                .map(|value| value as u32),
            zero_rpm: {
                if self.zero_rpm_switch.is_visible() {
                    Some(self.zero_rpm_switch.state())
                } else {
                    None
                }
            },
        }
    }
}

pub(super) fn set_fan_info(adjustment: &OcAdjustment, info: Option<FanInfo>) {
    match info {
        Some(info) => {
            if let Some((min, max)) = info.allowed_range {
                adjustment.set_lower(min as f64);
                adjustment.set_upper(max as f64);
            } else {
                adjustment.set_lower(0.0);
                adjustment.set_upper(info.current as f64);
            }

            adjustment.set_initial_value(info.current as f64);
        }
        None => {
            adjustment.set_upper(0.0);
            adjustment.set_initial_value(0.0);
        }
    }
}

fn adjustment(parent_grid: &Grid, label: &str, row: i32) -> OcAdjustment {
    let adjustment = OcAdjustment::new(0.0, 0.0, 100.0, 1.0, 1.0, 0.0);
    attach_adjustment(parent_grid, label, row, &adjustment);
    adjustment
}

pub(super) fn attach_adjustment(
    parent_grid: &Grid,
    label: &str,
    row: i32,
    adjustment: &OcAdjustment,
) {
    let label = Label::builder().label(label).halign(Align::Start).build();

    let scale = Scale::builder()
        .orientation(Orientation::Horizontal)
        .adjustment(adjustment)
        .hexpand(true)
        .margin_start(5)
        .margin_end(5)
        .build();

    let value_selector = SpinButton::new(Some(adjustment), 1.0, 0);
    let value_label = Label::new(None);

    let popover = Popover::builder().child(&value_selector).build();
    let value_button = MenuButton::builder()
        .popover(&popover)
        .child(&value_label)
        .build();

    adjustment.connect_value_changed(clone!(
        #[strong]
        value_label,
        #[strong]
        label,
        #[strong]
        scale,
        #[strong]
        value_button,
        move |adjustment| {
            let value = adjustment.value();
            value_label.set_text(&format!("{}", value as u32));

            if adjustment.upper() == 0.0 {
                label.hide();
                value_label.hide();
                scale.hide();
                value_button.hide();
            } else {
                label.show();
                value_label.show();
                scale.show();
                value_button.show();
            }
        }
    ));

    parent_grid.attach(&label, 0, row, 1, 1);
    parent_grid.attach(&scale, 1, row, 4, 1);
    parent_grid.attach(&value_button, 5, row, 1, 1);
}
