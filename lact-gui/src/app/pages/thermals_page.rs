mod fan_curve_frame;

use super::{
    oc_adjustment::OcAdjustment,
    oc_page::gpu_stats_section::{temperature_text, throttling_text},
    PageUpdate,
};
use crate::{
    app::{info_row::InfoRow, msg::AppMsg, page_section::PageSection},
    APP_BROKER, I18N,
};
use amdgpu_sysfs::gpu_handle::fan_control::FanInfo;
use fan_curve_frame::{
    CurveSetupMsg, FanCurveFrame, FanCurveFrameMsg, DEFAULT_SPEED_RANGE, DEFAULT_TEMP_RANGE,
};
use gtk::{
    glib::{
        self,
        object::{Cast, ObjectExt},
        SignalHandlerId,
    },
    prelude::{AdjustmentExt, BoxExt, ButtonExt, OrientableExt, RangeExt, ScaleExt, WidgetExt},
    Adjustment,
};
use i18n_embed_fl::fl;
use lact_daemon::AMDGPU_FAMILY_GC_11_0_0;
use lact_schema::{
    config::{FanControlSettings, FanCurve, GpuConfig},
    default_fan_curve, FanControlMode, SystemInfo,
};
use relm4::{
    binding::{Binding, BoolBinding, ConnectBinding, StringBinding},
    ComponentController, ComponentParts, ComponentSender, RelmObjectExt, RelmWidgetExt,
};
use std::{cell::Cell, rc::Rc};

const AUTO_PAGE: &str = "automatic";
const CURVE_PAGE: &str = "curve";
const STATIC_PAGE: &str = "static";

pub struct ThermalsPage {
    fan_curve_frame: relm4::Controller<FanCurveFrame>,
    system_info: SystemInfo,
    selected_mode: StringBinding,

    has_pmfw: bool,
    has_auto_threshold: bool,
    pmfw_options: PmfwOptions,
    pmfw_change_signals: Vec<(glib::Object, SignalHandlerId)>,

    temperatures: Option<String>,
    fan_speed: Option<String>,
    throttling: String,

    static_speed_adj: Adjustment,
}

#[derive(Clone, Default)]
struct PmfwOptions {
    target_temperature: OcAdjustment,
    acoustic_limit: OcAdjustment,
    acoustic_target: OcAdjustment,
    minimum_pwm: OcAdjustment,
    zero_rpm_temperature: OcAdjustment,
    zero_rpm_available: Rc<Cell<bool>>,
    zero_rpm: BoolBinding,
}

impl PmfwOptions {
    fn adjustments(&self) -> [&OcAdjustment; 5] {
        [
            &self.target_temperature,
            &self.acoustic_limit,
            &self.acoustic_target,
            &self.minimum_pwm,
            &self.zero_rpm_temperature,
        ]
    }

    fn is_empty(&self) -> bool {
        self.adjustments().iter().all(|adj| adj_is_empty(adj)) && !self.zero_rpm.get()
    }
}

#[derive(Debug)]
pub enum ThermalsPageMsg {
    Update { update: PageUpdate, initial: bool },
}

#[relm4::component(pub)]
impl relm4::Component for ThermalsPage {
    type Init = SystemInfo;
    type Input = ThermalsPageMsg;
    type Output = ();
    type CommandOutput = ();

    view! {
        gtk::ScrolledWindow {
            set_policy: (gtk::PolicyType::Never, gtk::PolicyType::Automatic),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 15,
                set_margin_horizontal: 20,

                gtk::Frame {
                    #[watch]
                    set_visible: model.system_info.amdgpu_overdrive_enabled == Some(false) && model.has_pmfw,

                    gtk::Label {
                        set_label: &fl!(I18N, "oc-missing-fan-control-warning"),
                    },
                },

                PageSection::new(&fl!(I18N, "monitoring-section")) {
                    append = &InfoRow {
                        set_name: fl!(I18N, "temperatures"),
                        #[watch]
                        set_value: model.temperatures.as_deref().unwrap_or("No sensors found"),
                    },

                    append = &InfoRow {
                        set_name: fl!(I18N, "fan-speed"),
                        #[watch]
                        set_value: model.fan_speed.as_deref().unwrap_or("No fan detected"),
                    },

                    append = &InfoRow {
                        set_name: fl!(I18N, "throttling"),
                        #[watch]
                        set_value: model.throttling.as_str(),
                    },
                },

                PageSection::new(&fl!(I18N, "fan-control-section")) {
                    // Disable fan configuration when overdrive is disabled on GPUs that have PMFW (RDNA3+)
                    #[watch]
                    set_sensitive: model.fan_speed.is_some() && !(model.system_info.amdgpu_overdrive_enabled == Some(false) && model.has_pmfw),

                    append = &gtk::StackSwitcher {
                        set_stack: Some(&stack),
                    },

                    #[name = "stack"]
                    append = &gtk::Stack {
                        #[watch]
                        set_visible: model.fan_speed.is_some(),

                        add_titled[Some(AUTO_PAGE), &fl!(I18N, "auto-page")] = &gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 5,

                            #[template]
                            FanSettingRow {
                                #[watch]
                                set_visible: !adj_is_empty(&model.pmfw_options.target_temperature),

                                #[template_child]
                                label {
                                    set_label: &fl!(I18N, "target-temp"),
                                    set_size_group: &label_size_group,
                                },

                                #[template_child]
                                scale {
                                    set_adjustment: &model.pmfw_options.target_temperature,
                                },

                                #[template_child]
                                spinbutton {
                                    set_adjustment: &model.pmfw_options.target_temperature,
                                    set_size_group: &spin_size_group,
                                },
                            },

                            #[template]
                            FanSettingRow {
                                #[watch]
                                set_visible: !adj_is_empty(&model.pmfw_options.acoustic_limit),

                                #[template_child]
                                label {
                                    set_label: &fl!(I18N, "acoustic-limit"),
                                    set_size_group: &label_size_group,
                                },

                                #[template_child]
                                scale {
                                    set_adjustment: &model.pmfw_options.acoustic_limit,
                                },

                                #[template_child]
                                spinbutton {
                                    set_adjustment: &model.pmfw_options.acoustic_limit,
                                    set_size_group: &spin_size_group,
                                },
                            },

                            #[template]
                            FanSettingRow {
                                #[watch]
                                set_visible: !adj_is_empty(&model.pmfw_options.acoustic_target),

                                #[template_child]
                                label {
                                    set_label: &fl!(I18N, "acoustic-target"),
                                    set_size_group: &label_size_group,
                                },

                                #[template_child]
                                scale {
                                    set_adjustment: &model.pmfw_options.acoustic_target,
                                },

                                #[template_child]
                                spinbutton {
                                    set_adjustment: &model.pmfw_options.acoustic_target,
                                    set_size_group: &spin_size_group,
                                },
                            },

                            #[template]
                            FanSettingRow {
                                #[watch]
                                set_visible: !adj_is_empty(&model.pmfw_options.minimum_pwm),

                                #[template_child]
                                label {
                                    set_label: &fl!(I18N, "min-fan-speed"),
                                    set_size_group: &label_size_group,
                                },

                                #[template_child]
                                scale {
                                    set_adjustment: &model.pmfw_options.minimum_pwm,
                                },

                                #[template_child]
                                spinbutton {
                                    set_adjustment: &model.pmfw_options.minimum_pwm,
                                    set_size_group: &spin_size_group,
                                },
                            },

                            gtk::Box {
                                set_orientation: gtk::Orientation::Horizontal,
                                set_spacing: 5,
                                #[watch]
                                set_visible: model.pmfw_options.zero_rpm_available.get(),

                                gtk::Label {
                                    set_label: &fl!(I18N, "zero-rpm"),
                                    set_xalign: 0.0,
                                    set_size_group: &label_size_group,
                                },

                                gtk::Switch {
                                    bind: &model.pmfw_options.zero_rpm,
                                    set_hexpand: true,
                                    set_halign: gtk::Align::End,
                                },
                            },

                            #[template]
                            FanSettingRow {
                                #[watch]
                                set_visible: !adj_is_empty(&model.pmfw_options.zero_rpm_temperature),

                                #[template_child]
                                label {
                                    set_label: &fl!(I18N, "zero-rpm-stop-temp"),
                                    set_size_group: &label_size_group,
                                },

                                #[template_child]
                                scale {
                                    set_adjustment: &model.pmfw_options.zero_rpm_temperature,
                                },

                                #[template_child]
                                spinbutton {
                                    set_adjustment: &model.pmfw_options.zero_rpm_temperature,
                                    set_size_group: &spin_size_group,
                                },
                            },

                            gtk::Button {
                                set_label: &fl!(I18N, "reset-button"),
                                set_halign: gtk::Align::End,
                                set_margin_vertical: 5,
                                set_tooltip_text: Some(&fl!(I18N, "pmfw-reset-warning")),
                                add_css_class: "destructive-action",
                                set_size_group: &spin_size_group,
                                #[watch]
                                set_visible: !model.pmfw_options.is_empty(),
                                connect_clicked => move |_| {
                                    APP_BROKER.send(AppMsg::ResetPmfw);
                                }
                            },
                        },
                        add_titled[Some(CURVE_PAGE), &fl!(I18N, "curve-page")] = model.fan_curve_frame.widget(),
                        add_titled[Some(STATIC_PAGE), &fl!(I18N, "static-page")] = &gtk::Box {
                            set_valign: gtk::Align::Start,
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 5,

                            #[template]
                            #[name = "static_speed_row"]
                            FanSettingRow {
                                #[template_child]
                                label {
                                    set_label: &fl!(I18N, "static-speed"),
                                },

                                #[template_child]
                                scale {
                                    set_adjustment: &model.static_speed_adj,
                                    connect_value_changed => move |_| {
                                        APP_BROKER.send(AppMsg::SettingsChanged);
                                    } @ static_speed_changed_signal,
                                },

                                #[template_child]
                                spinbutton {
                                    set_adjustment: &model.static_speed_adj,
                                },
                            },
                        },

                        add_binding: (&model.selected_mode, "visible-child-name"),
                        connect_visible_child_name_notify => move |_| {
                            APP_BROKER.send(AppMsg::SettingsChanged);
                        } @ mode_selected_signal,
                    },
                },
            }
        }
    }

    fn init(
        system_info: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let pmfw_options = PmfwOptions::default();

        for adj in pmfw_options.adjustments() {
            adj.set_step_increment(1.0);
            adj.set_page_increment(5.0);
        }

        let pmfw_change_signals = pmfw_options
            .adjustments()
            .into_iter()
            .map(|adj| {
                let signal = adj.connect_value_changed(|_| {
                    APP_BROKER.send(AppMsg::SettingsChanged);
                });
                (adj.clone().upcast(), signal)
            })
            .chain([(
                pmfw_options.zero_rpm.clone().upcast(),
                pmfw_options.zero_rpm.connect_value_notify(|_| {
                    APP_BROKER.send(AppMsg::SettingsChanged);
                }),
            )])
            .collect::<Vec<(glib::Object, SignalHandlerId)>>();

        let fan_curve_frame = FanCurveFrame::builder()
            .launch(pmfw_options.clone())
            .detach();

        let model = Self {
            fan_curve_frame,
            throttling: String::new(),
            temperatures: None,
            system_info,
            pmfw_options,
            pmfw_change_signals,
            has_pmfw: false,
            has_auto_threshold: false,
            fan_speed: None,
            static_speed_adj: Adjustment::new(50.0, 0.0, 100.0, 1.0, 5.0, 0.0),
            selected_mode: StringBinding::new(AUTO_PAGE),
        };

        let label_size_group = gtk::SizeGroup::new(gtk::SizeGroupMode::Horizontal);
        let spin_size_group = gtk::SizeGroup::new(gtk::SizeGroupMode::Horizontal);

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
                    self.has_pmfw = info
                        .drm_info
                        .as_ref()
                        .and_then(|info| info.family_id)
                        .is_some_and(|family| family >= AMDGPU_FAMILY_GC_11_0_0);

                    self.has_auto_threshold = info.driver.starts_with("nvidia ");
                }
                PageUpdate::Stats(stats) => {
                    let fan_percent = stats
                        .fan
                        .pwm_current
                        .map(|current_pwm| ((current_pwm as f64 / u8::MAX as f64) * 100.0).round());
                    let fan_text = if let Some(current_rpm) = stats.fan.speed_current {
                        let text = match fan_percent {
                            Some(percent) => format!("<b>{current_rpm} RPM ({percent}%)</b>",),
                            None => format!("<b>{current_rpm} RPM</b>"),
                        };
                        Some(text)
                    } else {
                        fan_percent.map(|percent| format!("<b>{percent}%</b>"))
                    };

                    self.fan_speed = fan_text;
                    self.temperatures = temperature_text(&stats);
                    self.throttling = throttling_text(&stats);

                    if initial {
                        let page_name = match stats.fan.control_mode {
                            Some(mode) if stats.fan.control_enabled => match mode {
                                FanControlMode::Static => STATIC_PAGE,
                                FanControlMode::Curve => CURVE_PAGE,
                            },
                            _ => AUTO_PAGE,
                        };

                        widgets.stack.block_signal(&widgets.mode_selected_signal);
                        self.selected_mode.set(page_name.to_owned());
                        widgets.stack.unblock_signal(&widgets.mode_selected_signal);

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

                        widgets
                            .static_speed_row
                            .scale
                            .block_signal(&widgets.static_speed_changed_signal);

                        self.static_speed_adj
                            .set_lower((*speed_range.start() as f64 * 100.0).round());
                        self.static_speed_adj
                            .set_upper((*speed_range.end() as f64 * 100.0).round());
                        self.static_speed_adj
                            .set_value((stats.fan.static_speed.unwrap_or(0.5) * 100.0).into());

                        widgets
                            .static_speed_row
                            .scale
                            .unblock_signal(&widgets.static_speed_changed_signal);

                        let temperature_range = stats
                            .fan
                            .temperature_range
                            .map(|(start, end)| start as f32..=end as f32)
                            .unwrap_or(DEFAULT_TEMP_RANGE);

                        let msg = CurveSetupMsg {
                            curve: stats.fan.curve.clone().unwrap_or_else(default_fan_curve),
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
                        let pmfw_options = &mut self.pmfw_options;

                        for (obj, signal) in &self.pmfw_change_signals {
                            obj.block_signal(signal);
                        }

                        set_fan_info(&pmfw_options.acoustic_limit, info.acoustic_limit);
                        set_fan_info(&pmfw_options.acoustic_target, info.acoustic_target);
                        set_fan_info(&pmfw_options.minimum_pwm, info.minimum_pwm);
                        set_fan_info(&pmfw_options.target_temperature, info.target_temp);
                        set_fan_info(
                            &pmfw_options.zero_rpm_temperature,
                            info.zero_rpm_temperature,
                        );

                        pmfw_options
                            .zero_rpm_available
                            .set(info.zero_rpm_enable.is_some());
                        pmfw_options
                            .zero_rpm
                            .set(info.zero_rpm_enable.unwrap_or(false));

                        for (obj, signal) in &self.pmfw_change_signals {
                            obj.unblock_signal(signal);
                        }
                    }
                }
            },
        }

        self.update_view(widgets, sender);
    }
}

impl ThermalsPage {
    pub fn apply_config(&self, config: &mut GpuConfig) {
        let selected_page = self.selected_mode.value();

        if selected_page == AUTO_PAGE {
            config.fan_control_enabled = false;
        } else {
            config.fan_control_enabled = true;
            let fan_settings = config
                .fan_control_settings
                .get_or_insert_with(FanControlSettings::default);

            match selected_page.as_str() {
                CURVE_PAGE => {
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
                STATIC_PAGE => {
                    fan_settings.mode = FanControlMode::Static;
                    fan_settings.static_speed = self.static_speed_adj.value() as f32 / 100.0;
                }
                _ => unreachable!("Invalid fan control page selected"),
            }
        }

        let pmfw = &self.pmfw_options;
        let config = &mut config.pmfw_options;

        let options = [
            (&pmfw.acoustic_limit, &mut config.acoustic_limit),
            (&pmfw.acoustic_target, &mut config.acoustic_target),
            (&pmfw.target_temperature, &mut config.target_temperature),
            (&pmfw.minimum_pwm, &mut config.minimum_pwm),
            (&pmfw.zero_rpm_temperature, &mut config.zero_rpm_threshold),
        ];

        for (adj, config_value) in options {
            if let Some(value) = adj.get_changed_value(false) {
                *config_value = Some(value as u32);
            }
        }

        if pmfw.zero_rpm_available.get() {
            config.zero_rpm = Some(pmfw.zero_rpm.value());
        }
    }
}

#[relm4::widget_template(pub)]
impl relm4::WidgetTemplate for FanSettingRow {
    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: 5,

            #[name = "label"]
            gtk::Label {
                set_xalign: 0.0,
            },

            #[name = "scale"]
            gtk::Scale {
                set_orientation: gtk::Orientation::Horizontal,
                set_hexpand: true,
                set_digits: 0,
                set_round_digits: 0,
                set_value_pos: gtk::PositionType::Right,
                set_margin_horizontal: 5,
            },

            #[name = "spinbutton"]
            gtk::SpinButton {},
        },
    }
}

fn adj_is_empty(adj: &OcAdjustment) -> bool {
    adj.lower() == 0.0 && adj.upper() == 0.0
}

fn set_fan_info(adjustment: &OcAdjustment, info: Option<FanInfo>) {
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
