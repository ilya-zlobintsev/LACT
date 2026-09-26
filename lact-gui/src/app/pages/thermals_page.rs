mod fan_curve_frame;

use crate::app::components::gpu_stats_section::{
    GpuStat, GpuStatsSection, GpuStatsSectionConfig, GpuStatsSectionMsg,
};
use crate::app::pages::PageUpdate;
use crate::{
    APP_BROKER, I18N,
    app::{
        components::{
            adjustment_card::AdjustmentCard,
            adjustment_row::{AdjustmentRow, AdjustmentRowInit, AdjustmentRowMsg},
            page_section::PageSection,
        },
        msg::AppMsg,
        utils::ext::RelmLaunchable as _,
    },
};
use adw::prelude::*;
use amdgpu_sysfs::gpu_handle::fan_control::FanInfo;
use anyhow::anyhow;
use fan_curve_frame::{
    CurveSetupMsg, DEFAULT_SPEED_RANGE, DEFAULT_TEMP_RANGE, FanCurveFrame, FanCurveFrameMsg,
};
use gtk::glib::{self, SignalHandlerId};
use i18n_embed_fl::fl;
use lact_schema::{
    DeviceFlag, FanControlMode,
    config::{FanControlSettings, FanCurve, GpuConfig},
    default_fan_curve,
};
use relm4::{
    ComponentController, ComponentParts, ComponentSender, RelmWidgetExt, WidgetTemplate,
    binding::{Binding, BoolBinding, ConnectBinding},
    factory::FactoryHashMap,
};
use std::collections::HashSet;
use std::sync::Arc;

pub struct ThermalsPage {
    stats_section: relm4::Controller<GpuStatsSection>,
    pmfw_rows: FactoryHashMap<ThermalSetting, AdjustmentRow<ThermalSetting>>,
    nvidia_target_temperature: FactoryHashMap<(), AdjustmentRow<()>>,
    zero_rpm_temperature: FactoryHashMap<(), AdjustmentRow<()>>,
    static_speed: FactoryHashMap<(), AdjustmentRow<()>>,
    fan_curve_frame: relm4::Controller<FanCurveFrame>,
    selected_mode: Option<FanControlMode>,

    custom_control_supported: bool,
    has_fan_speed: bool,
    has_pmfw: bool,
    has_auto_threshold: bool,
    label_size_group: gtk::SizeGroup,
    input_size_group: gtk::SizeGroup,
    zero_rpm: BoolBinding,
    zero_rpm_available: bool,
    zero_rpm_change_signal: SignalHandlerId,
    target_temperature_default: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThermalSetting {
    TargetTemperature,
    AcousticLimit,
    AcousticTarget,
    MinimumPwm,
}

#[derive(Debug)]
pub enum ThermalsPageMsg {
    Update { update: PageUpdate, initial: bool },
    FanModeSelected(Option<FanControlMode>),
    RestNvidiaOptions,
}

#[relm4::component(pub)]
impl relm4::Component for ThermalsPage {
    type Init = ();
    type Input = ThermalsPageMsg;
    type Output = ();
    type CommandOutput = ();

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            add_css_class: "thermals-page",
            set_spacing: 15,
            set_margin_all: 15,
            set_margin_top: 20, // align with gpu picker

            model.stats_section.widget(),

            PageSection {
                set_name: fl!(I18N, "thresholds-section"),
                set_hide_visible_container: true,
                #[watch]
                set_visible: !model.nvidia_target_temperature.is_empty(),

                append_header = &gtk::Button {
                    set_label: &fl!(I18N, "default-button"),
                    set_halign: gtk::Align::End,
                    set_hexpand: true,
                    connect_clicked => ThermalsPageMsg::RestNvidiaOptions,
                },

                append_child = model.nvidia_target_temperature.widget(),
            },

            PageSection {
                set_name: fl!(I18N, "fan-control-section"),
                add_css_class: "fan-control-section",
                // Disable fan configuration when overdrive is disabled on GPUs that have PMFW (RDNA3+)
                #[watch]
                set_sensitive: model.custom_control_supported,

                append_child = &gtk::Box {
                    add_css_class: "linked",
                    set_homogeneous: true,

                    #[name = "automatic_mode"]
                    gtk::ToggleButton {
                        set_label: &fl!(I18N, "auto-page"),
                        #[watch]
                        #[block_signal(automatic_mode_signal)]
                        set_active: model.selected_mode.is_none(),
                        connect_toggled[sender] => move |button| {
                            if button.is_active() {
                                sender.input(ThermalsPageMsg::FanModeSelected(None));
                            }
                        } @ automatic_mode_signal,
                    },

                    gtk::ToggleButton {
                        set_label: &fl!(I18N, "curve-page"),
                        set_group: Some(&automatic_mode),
                        #[watch]
                        #[block_signal(curve_mode_signal)]
                        set_active: model.selected_mode == Some(FanControlMode::Curve),
                        connect_toggled[sender] => move |button| {
                            if button.is_active() {
                                sender.input(ThermalsPageMsg::FanModeSelected(Some(FanControlMode::Curve)));
                            }
                        } @ curve_mode_signal,
                    },

                    gtk::ToggleButton {
                        set_label: &fl!(I18N, "static-page"),
                        set_group: Some(&automatic_mode),
                        #[watch]
                        #[block_signal(static_mode_signal)]
                        set_active: model.selected_mode == Some(FanControlMode::Static),
                        connect_toggled[sender] => move |button| {
                            if button.is_active() {
                                sender.input(ThermalsPageMsg::FanModeSelected(Some(FanControlMode::Static)));
                            }
                        } @ static_mode_signal,
                    },
                },

                append_child = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 5,
                    #[watch]
                    set_visible: model.fan_settings_available(),

                    model.pmfw_rows.widget().clone() -> gtk::ListBox {
                        #[watch]
                        set_visible: model.selected_mode.is_none() && !model.pmfw_rows.is_empty(),
                    },

                    model.fan_curve_frame.widget().clone() -> gtk::Box {
                        #[watch]
                        set_visible: model.selected_mode == Some(FanControlMode::Curve),
                    },

                    model.static_speed.widget().clone() -> gtk::ListBox {
                        #[watch]
                        set_visible: model.selected_mode == Some(FanControlMode::Static),
                    },

                    gtk::Box {
                        add_css_class: "fan-control-option",
                        set_spacing: 5,
                        #[watch]
                        set_visible: model.selected_mode != Some(FanControlMode::Static)
                            && model.zero_rpm_available,

                        gtk::Label {
                            set_label: &fl!(I18N, "zero-rpm"),
                            set_size_group: &model.label_size_group,
                            set_xalign: 0.0,
                        },

                        gtk::Switch {
                            bind: &model.zero_rpm,
                            set_hexpand: true,
                            set_halign: gtk::Align::End,
                        },
                    },

                    model.zero_rpm_temperature.widget().clone() -> gtk::ListBox {
                        #[watch]
                        set_visible: model.selected_mode != Some(FanControlMode::Static)
                            && !model.zero_rpm_temperature.is_empty(),
                    },

                    gtk::Button {
                        set_label: &fl!(I18N, "reset-now-button"),
                        set_size_group: &model.input_size_group,
                        set_halign: gtk::Align::End,
                        set_margin_vertical: 5,
                        set_tooltip_text: Some(&fl!(I18N, "pmfw-reset-warning")),
                        add_css_class: "destructive-action",
                        #[watch]
                        set_visible: model.selected_mode.is_none() && model.has_pmfw_options(),
                        connect_clicked => move |_| {
                            APP_BROKER.send(AppMsg::ResetPmfw);
                        }
                    },
                }
            }
        },
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let zero_rpm = BoolBinding::new(false);
        let zero_rpm_change_signal = zero_rpm.connect_value_notify(|_| {
            APP_BROKER.send(AppMsg::SettingsChanged);
        });
        let fan_curve_frame = FanCurveFrame::detach(());
        let stats_section = GpuStatsSection::detach(GpuStatsSectionConfig {
            stats: HashSet::from([
                GpuStat::Throttling,
                GpuStat::Temperature,
                GpuStat::PowerUsage,
                GpuStat::FanSpeed,
            ]),
        });

        let model = Self {
            pmfw_rows: FactoryHashMap::builder()
                .launch(AdjustmentCard::init(()).content.clone())
                .forward(APP_BROKER.sender(), |()| AppMsg::SettingsChanged),
            nvidia_target_temperature: FactoryHashMap::builder()
                .launch(AdjustmentCard::init(()).content.clone())
                .forward(APP_BROKER.sender(), |()| AppMsg::SettingsChanged),
            zero_rpm_temperature: FactoryHashMap::builder()
                .launch(AdjustmentCard::init(()).content.clone())
                .forward(APP_BROKER.sender(), |()| AppMsg::SettingsChanged),
            static_speed: FactoryHashMap::builder()
                .launch(AdjustmentCard::init(()).content.clone())
                .forward(APP_BROKER.sender(), |()| AppMsg::SettingsChanged),
            stats_section,
            fan_curve_frame,
            label_size_group: gtk::SizeGroup::new(gtk::SizeGroupMode::Horizontal),
            input_size_group: gtk::SizeGroup::new(gtk::SizeGroupMode::Horizontal),
            zero_rpm,
            zero_rpm_available: false,
            zero_rpm_change_signal,
            target_temperature_default: None,
            custom_control_supported: false,
            has_fan_speed: false,
            has_pmfw: false,
            has_auto_threshold: false,
            selected_mode: None,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            ThermalsPageMsg::Update { update, initial } => match update {
                PageUpdate::Info(info) => {
                    self.stats_section
                        .emit(GpuStatsSectionMsg::Info(info.clone()));

                    self.custom_control_supported =
                        info.flags.contains(&DeviceFlag::ConfigurableFanControl);
                    self.has_pmfw = info.flags.contains(&DeviceFlag::HasPmfw);
                    self.has_auto_threshold = info.flags.contains(&DeviceFlag::AutoFanThreshold);
                }
                PageUpdate::Stats(stats) => {
                    self.stats_section
                        .emit(GpuStatsSectionMsg::Stats(stats.clone()));

                    self.has_fan_speed =
                        stats.fan.pwm_current.is_some() || stats.fan.speed_current.is_some();
                    if initial {
                        self.selected_mode =
                            stats.fan.control_mode.filter(|_| stats.fan.control_enabled);

                        let speed_range = stats
                            .fan
                            .pwm_min
                            .zip(stats.fan.pwm_max)
                            .map(|(min, max)| {
                                let min = min as f32 / f32::from(u8::MAX);
                                let max = max as f32 / f32::from(u8::MAX);
                                min..=max
                            })
                            .unwrap_or(DEFAULT_SPEED_RANGE);

                        self.pmfw_rows.clear();
                        self.nvidia_target_temperature.clear();
                        self.zero_rpm_temperature.clear();
                        self.static_speed.clear();

                        self.static_speed.insert(
                            (),
                            AdjustmentRowInit {
                                title: glib::markup_escape_text(&fl!(I18N, "static-speed")).into(),
                                value: (stats.fan.static_speed.unwrap_or(0.5) * 100.0).into(),
                                lower: (*speed_range.start() as f64 * 100.0).round(),
                                upper: (*speed_range.end() as f64 * 100.0).round(),
                                page_increment: 5.0,
                                ..Default::default()
                            },
                        );

                        let temperature_range = stats
                            .fan
                            .temperature_range
                            .map(|(start, end)| start as f32..=end as f32)
                            .unwrap_or(DEFAULT_TEMP_RANGE);

                        let msg = CurveSetupMsg {
                            curve: stats.fan.curve.clone().unwrap_or_else(default_fan_curve),
                            hw_based: self.has_pmfw,
                            current_temperatures: stats.temps.clone(),
                            temperature_key: stats.fan.temperature_key.clone(),
                            spindown_delay: stats.fan.spindown_delay_ms,
                            change_threshold: stats.fan.change_threshold,
                            speed_range,
                            temperature_range,
                            auto_threshold_supported: self.has_auto_threshold,
                            auto_threshold: stats.fan.auto_threshold,
                        };
                        self.fan_curve_frame.emit(FanCurveFrameMsg::Curve(msg));

                        let info = stats.fan.pmfw_info;
                        let lower_label_group = gtk::SizeGroup::new(gtk::SizeGroupMode::Horizontal);
                        let upper_label_group = gtk::SizeGroup::new(gtk::SizeGroupMode::Horizontal);
                        for (setting, title, info) in [
                            (
                                ThermalSetting::TargetTemperature,
                                fl!(I18N, "target-temp"),
                                info.target_temp,
                            ),
                            (
                                ThermalSetting::AcousticLimit,
                                fl!(I18N, "acoustic-limit"),
                                info.acoustic_limit,
                            ),
                            (
                                ThermalSetting::AcousticTarget,
                                fl!(I18N, "acoustic-target"),
                                info.acoustic_target,
                            ),
                            (
                                ThermalSetting::MinimumPwm,
                                fl!(I18N, "min-fan-speed"),
                                info.minimum_pwm,
                            ),
                        ] {
                            if let Some(init) = fan_info_init(title, info) {
                                self.pmfw_rows.insert(setting, init);
                                self.pmfw_rows.send(
                                    &setting,
                                    AdjustmentRowMsg::AddSizeGroup {
                                        label_group: self.label_size_group.clone(),
                                        input_group: self.input_size_group.clone(),
                                        lower_label_group: lower_label_group.clone(),
                                        upper_label_group: upper_label_group.clone(),
                                    },
                                );
                            }
                        }
                        for (rows, title, info) in [
                            (
                                &mut self.nvidia_target_temperature,
                                fl!(I18N, "target-temp"),
                                stats.nvidia_thermal_info.target_temp,
                            ),
                            (
                                &mut self.zero_rpm_temperature,
                                fl!(I18N, "zero-rpm-stop-temp"),
                                info.zero_rpm_temperature,
                            ),
                        ] {
                            if let Some(init) = fan_info_init(title, info) {
                                rows.insert((), init);
                                rows.send(
                                    &(),
                                    AdjustmentRowMsg::AddSizeGroup {
                                        label_group: self.label_size_group.clone(),
                                        input_group: self.input_size_group.clone(),
                                        lower_label_group: lower_label_group.clone(),
                                        upper_label_group: upper_label_group.clone(),
                                    },
                                );
                            }
                        }
                        self.target_temperature_default =
                            stats.nvidia_thermal_info.target_temp_default;
                        self.zero_rpm_available = info.zero_rpm_enable.is_some();
                        self.zero_rpm.block_signal(&self.zero_rpm_change_signal);
                        self.zero_rpm.set(info.zero_rpm_enable.unwrap_or(false));
                        self.zero_rpm.unblock_signal(&self.zero_rpm_change_signal);
                    }
                }
            },
            ThermalsPageMsg::FanModeSelected(mode) => {
                self.selected_mode = mode;
                APP_BROKER.send(AppMsg::SettingsChanged);
            }
            ThermalsPageMsg::RestNvidiaOptions => {
                if let Some(default) = self.target_temperature_default {
                    if !self.nvidia_target_temperature.is_empty() {
                        self.nvidia_target_temperature
                            .send(&(), AdjustmentRowMsg::SetValue(default as f64));
                    }
                } else {
                    APP_BROKER.send(AppMsg::Error(Arc::new(anyhow!(
                        "No default target temperature present"
                    ))));
                }
            }
        }

        self.update_view(widgets, sender);
    }
}

impl ThermalsPage {
    fn fan_settings_available(&self) -> bool {
        self.has_fan_speed && (self.selected_mode.is_some() || self.has_pmfw_options())
    }

    pub fn apply_config(&self, config: &mut GpuConfig) {
        if let Some(mode) = self.selected_mode {
            config.fan_control_enabled = true;
            let fan_settings = config
                .fan_control_settings
                .get_or_insert_with(FanControlSettings::default);

            match mode {
                FanControlMode::Curve => {
                    fan_settings.mode = FanControlMode::Curve;

                    let fan_curve_model = self.fan_curve_frame.model();
                    fan_settings.curve = FanCurve(fan_curve_model.get_curve());
                    fan_settings.change_threshold = Some(fan_curve_model.change_threshold());
                    fan_settings.spindown_delay_ms = Some(fan_curve_model.spindown_delay());

                    if let Some(threshold) = fan_curve_model.auto_threshold() {
                        fan_settings.auto_threshold = Some(threshold);
                    }

                    if let Some(temp_key) = fan_curve_model.temperature_key() {
                        fan_settings.temperature_key = temp_key;
                    }
                }
                FanControlMode::Static => {
                    fan_settings.mode = FanControlMode::Static;
                    fan_settings.static_speed = self
                        .static_speed
                        .get(&())
                        .map(|row| row.get_value() as f32 / 100.0)
                        .unwrap_or(0.5);
                }
            }
        } else {
            config.fan_control_enabled = false;
        }

        let pmfw_config = &mut config.pmfw_options;
        for (setting, config_value) in [
            (
                ThermalSetting::AcousticLimit,
                &mut pmfw_config.acoustic_limit,
            ),
            (
                ThermalSetting::AcousticTarget,
                &mut pmfw_config.acoustic_target,
            ),
            (
                ThermalSetting::TargetTemperature,
                &mut pmfw_config.target_temperature,
            ),
            (ThermalSetting::MinimumPwm, &mut pmfw_config.minimum_pwm),
        ] {
            if let Some(value) = self
                .pmfw_rows
                .get(&setting)
                .and_then(|row| row.get_changed_value())
            {
                *config_value = Some(value as u32);
            }
        }
        if let Some(value) = self
            .zero_rpm_temperature
            .get(&())
            .and_then(|row| row.get_changed_value())
        {
            pmfw_config.zero_rpm_threshold = Some(value as u32);
        }
        if let Some(value) = self
            .nvidia_target_temperature
            .get(&())
            .and_then(|row| row.get_changed_value())
        {
            config.nvidia_thermal_options.target_temperature = Some(value as u32);
        }
        if self.zero_rpm_available {
            pmfw_config.zero_rpm = Some(self.zero_rpm.value());
        }
    }

    fn has_pmfw_options(&self) -> bool {
        self.zero_rpm_available
            || !self.pmfw_rows.is_empty()
            || !self.zero_rpm_temperature.is_empty()
    }
}

fn fan_info_init(title: String, info: Option<FanInfo>) -> Option<AdjustmentRowInit> {
    let info = info?;
    let (min, max) = info.allowed_range.unwrap_or((0, info.current));
    (min != 0 || max != 0).then(|| AdjustmentRowInit {
        title: glib::markup_escape_text(&title).into(),
        value: info.current as f64,
        lower: min as f64,
        upper: max as f64,
        page_increment: 5.0,
        ..Default::default()
    })
}
