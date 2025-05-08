mod fan_curve_frame;
mod pmfw_frame;

use super::{oc_page::gpu_stats_section::throttling_text, PageUpdate};
use crate::{
    app::{ext::RelmDefaultLauchable, info_row::InfoRow, msg::AppMsg, page_section::PageSection},
    APP_BROKER,
};
use fan_curve_frame::{FanCurveFrame, FanCurveFrameMsg, DEFAULT_SPEED_RANGE, DEFAULT_TEMP_RANGE};
use gtk::{
    glib::object::ObjectExt,
    prelude::{BoxExt, OrientableExt},
};
use lact_schema::{
    config::{FanControlSettings, FanCurve, GpuConfig},
    default_fan_curve, FanControlMode,
};
use relm4::{
    binding::{Binding, StringBinding},
    ComponentController, ComponentParts, ComponentSender, RelmObjectExt, RelmWidgetExt,
};

const AUTO_PAGE: &str = "automatic";
const CURVE_PAGE: &str = "curve";
const STATIC_PAGE: &str = "static";

pub struct ThermalsPage {
    fan_curve_frame: relm4::Controller<FanCurveFrame>,

    temperatures: Option<String>,
    fan_speed: Option<String>,
    throttling: String,

    selected_mode: StringBinding,
}

#[derive(Debug)]
pub enum ThermalsPageMsg {
    Update { update: PageUpdate, initial: bool },
}

#[relm4::component(pub)]
impl relm4::Component for ThermalsPage {
    type Init = ();
    type Input = ThermalsPageMsg;
    type Output = ();
    type CommandOutput = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 15,
            set_margin_horizontal: 20,

            PageSection::new("Monitoring") {
                append = &InfoRow {
                    set_name: "Temperatures:",
                    #[watch]
                    set_value: model.temperatures.as_deref().unwrap_or("No sensors found"),
                },

                append = &InfoRow {
                    set_name: "Fan Speed:",
                    #[watch]
                    set_value: model.fan_speed.as_deref().unwrap_or("No fan detected"),
                },

                append = &InfoRow {
                    set_name: "Throttling:",
                    #[watch]
                    set_value: model.throttling.as_str(),
                },
            },

            PageSection::new("Fan Control") {
                append = &gtk::StackSwitcher {
                    set_stack: Some(&stack),
                },

                #[name = "stack"]
                append = &gtk::Stack {
                    add_titled[Some(AUTO_PAGE), "Automatic"] = &gtk::Box {

                    },
                    add_titled[Some(CURVE_PAGE), "Curve"] = model.fan_curve_frame.widget(),
                    add_titled[Some(STATIC_PAGE), "Static"] = &gtk::Box {

                    },

                    add_binding: (&model.selected_mode, "visible-child-name"),
                    connect_visible_child_name_notify => move |_| {
                        APP_BROKER.send(AppMsg::SettingsChanged);
                    } @ mode_selected_signal,
                },
            },
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let fan_curve_frame = FanCurveFrame::detach_default();

        let model = Self {
            fan_curve_frame,
            throttling: String::new(),
            temperatures: None,
            fan_speed: None,
            selected_mode: StringBinding::new(AUTO_PAGE),
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
                PageUpdate::Info(_device_info) => (),
                PageUpdate::Stats(stats) => {
                    let mut temperatures: Vec<String> = stats
                        .temps
                        .iter()
                        .filter_map(|(label, temp)| {
                            temp.current.map(|current| format!("{label}: {current}°C"))
                        })
                        .collect();
                    temperatures.sort_unstable();

                    self.temperatures = if temperatures.is_empty() {
                        None
                    } else {
                        Some(temperatures.join(", "))
                    };

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

                        let temperature_range = stats
                            .fan
                            .temperature_range
                            .map(|(start, end)| start as f32..=end as f32)
                            .unwrap_or(DEFAULT_TEMP_RANGE);

                        self.fan_curve_frame.emit(FanCurveFrameMsg::Curve {
                            curve: stats.fan.curve.clone().unwrap_or_else(default_fan_curve),
                            speed_range,
                            temperature_range,
                        });
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
                    fan_settings.curve = FanCurve(self.fan_curve_frame.model().get_curve());
                }
                STATIC_PAGE => {
                    fan_settings.mode = FanControlMode::Static;
                }
                _ => unreachable!("Invalid fan control page selected"),
            }
        }
    }
}
