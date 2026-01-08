use crate::app::msg::AppMsg;
use crate::app::APP_BROKER;
use crate::app::{
    ext::FlowBoxExt, formatting::Mono, info_row::InfoRow, page_section::PageSection,
    pages::PageUpdate,
};
use crate::I18N;
use gtk::prelude::{BoxExt, ButtonExt, OrientableExt, WidgetExt};
use i18n_embed_fl::fl;
use lact_schema::{DeviceStats, PowerStats};
use relm4::{ComponentParts, ComponentSender};
use std::{fmt::Write, sync::Arc};

pub struct GpuStatsSection {
    stats: Arc<DeviceStats>,
    vram_clock_ratio: f64,
    gpu_model: String,
}

#[relm4::component(pub)]
impl relm4::SimpleComponent for GpuStatsSection {
    type Init = ();
    type Input = PageUpdate;
    type Output = ();

    view! {
        PageSection::new(&fl!(I18N, "stats-section")) {
            append = &gtk::FlowBox {
                set_orientation: gtk::Orientation::Horizontal,
                set_column_spacing: 10,
                set_homogeneous: true,
                set_min_children_per_line: 2,
                set_selection_mode: gtk::SelectionMode::None,
                add_css_class: "card",

                append = &InfoRow {
                    set_name: fl!(I18N, "device-name"),
                    #[watch]
                    set_value: model.gpu_model.clone(),
                },

                append = &InfoRow {
                    set_name: fl!(I18N, "throttling"),
                    #[watch]
                    set_value: throttling_text(&model.stats),
                },

                append_child = &InfoRow {
                    #[watch]
                    set_name: {
                        if model.stats.clockspeed.gpu_clockspeed.is_some()
                            && model.stats.clockspeed.target_gpu_clockspeed.is_some() {
                                fl!(I18N, "gpu-clock-avg")
                            } else {
                                fl!(I18N, "gpu-clock")
                            }
                    },
                    #[watch]
                    set_value: format_clockspeed(model.stats.clockspeed.gpu_clockspeed, 1.0),
                } -> clockspeed_item: gtk::FlowBoxChild {
                    #[watch]
                    set_visible: model.stats.clockspeed.gpu_clockspeed.is_some(),
                },

                append_child = &InfoRow {
                    set_name: fl!(I18N, "gpu-clock-target"),
                    #[watch]
                    set_value: format_current_gfxclk(model.stats.clockspeed.target_gpu_clockspeed),
                } -> clockspeed_target_item: gtk::FlowBoxChild {
                    #[watch]
                    set_visible: model.stats.clockspeed.target_gpu_clockspeed.is_some(),
                },

                append_child = &InfoRow {
                    set_name: fl!(I18N, "vram-clock"),
                    #[watch]
                    set_value: format_clockspeed(
                        model.stats.clockspeed.vram_clockspeed,
                        model.vram_clock_ratio,
                    ),
                } -> vram_clock_item: gtk::FlowBoxChild {
                    #[watch]
                    set_visible: model.stats.clockspeed.vram_clockspeed.is_some(),
                },

                append_child = &InfoRow {
                    set_name: fl!(I18N, "gpu-voltage"),
                    #[watch]
                    set_value: format!("{} V", Mono::float(model.stats.voltage.gpu.unwrap_or(0) as f64 / 1000f64, 3)),
                } -> gpu_voltage_item: gtk::FlowBoxChild {
                    #[watch]
                    set_visible: model.stats.voltage.gpu.is_some(),
                },

                append_child = &InfoRow {
                    set_name: fl!(I18N, "power-usage"),
                    #[watch]
                    set_value: {
                        let PowerStats {
                            average: power_average,
                            current: power_current,
                            cap_current: power_cap_current,
                            ..
                        } = model.stats.power;

                        let power_current = power_current
                            .filter(|value| *value != 0.0)
                            .or(power_average);

                        if let Some(cap) = power_cap_current {
                            format!(
                                "<b>{}/{} {}</b>",
                                Mono::float(power_current.unwrap_or(0.0), 1),
                                Mono::float(cap, 0),
                                fl!(I18N, "watt")
                            )
                        } else {
                            format!(
                                "<b>{} {}</b>",
                                Mono::float(power_current.unwrap_or(0.0), 1),
                                fl!(I18N, "watt")
                            )
                        }

                    },
                } -> power_usage_item: gtk::FlowBoxChild {
                    #[watch]
                    set_visible: model.stats.power.average.is_some() || model.stats.power.current.is_some(),
                },

                append_child = &InfoRow {
                    set_name: fl!(I18N, "gpu-usage"),
                    #[watch]
                    set_value: format!("{}%", Mono::uint(model.stats.busy_percent.unwrap_or(0))),
                } -> gpu_usage_item: gtk::FlowBoxChild {
                    #[watch]
                    set_visible: model.stats.busy_percent.is_some(),
                },


                append = &gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 5,

                    gtk::Label {
                        set_label: &fl!(I18N, "vram-usage"),
                    },

                    gtk::Overlay {
                        gtk::LevelBar {
                            set_hexpand: true,
                            set_orientation: gtk::Orientation::Horizontal,
                            #[watch]
                            set_value: model
                                .stats
                                .vram
                                .used
                                .zip(model.stats.vram.total)
                                .map(|(used, total)| used as f64 / total as f64)
                                .unwrap_or(0.0),
                        },

                        add_overlay = &gtk::Label {
                            #[watch]
                            set_label: &format!(
                                "{}/{} {}",
                                model.stats.vram.used.unwrap_or(0) / 1024 / 1024,
                                model.stats.vram.total.unwrap_or(0) / 1024 / 1024,
                                fl!(I18N, "mebibyte")
                            ),
                        }
                    },
                },

                append = &InfoRow {
                    set_name: fl!(I18N, "gpu-temp"),
                    #[watch]
                    set_value: temperature_text(&model.stats).unwrap_or_else(|| "N/A".to_owned()),
                },

                append_child = &InfoRow {
                    set_name: fl!(I18N, "fan-speed"),
                    #[watch]
                    set_value: fan_speed_text(&model.stats).unwrap_or_else(|| fl!(I18N, "missing-stat")),
                } -> fan_speed_item: gtk::FlowBoxChild {
                    #[watch]
                    set_visible: model.stats.fan.pwm_current.is_some() || model.stats.fan.speed_current.is_some(),
                },
            },

            append = &gtk::Button {
                set_label: &fl!(I18N, "show-historical-charts"),
                connect_clicked => move |_| APP_BROKER.send(AppMsg::ShowGraphsWindow),
            },
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            stats: Arc::new(DeviceStats::default()),
            vram_clock_ratio: 1.0,
            gpu_model: String::new(),
        };

        let widgets = view_output!();

        ComponentParts { widgets, model }
    }

    fn update(&mut self, msg: Self::Input, _sender: relm4::ComponentSender<Self>) {
        match msg {
            PageUpdate::Info(info) => {
                self.vram_clock_ratio = info.vram_clock_ratio();
                if let Some(pci_info) = &info.pci_info {
                    self.gpu_model = info
                        .drm_info
                        .as_ref()
                        .and_then(|drm| drm.device_name.as_deref())
                        .or_else(|| pci_info.device_pci_info.model.as_deref())
                        .unwrap_or("Unknown")
                        .to_owned();
                }
            }
            PageUpdate::Stats(stats) => {
                self.stats = stats;
            }
        }
    }
}

fn format_clockspeed(value: Option<u64>, ratio: f64) -> String {
    format!(
        "{} {}",
        Mono::float(value.unwrap_or(0) as f64 / 1000.0 * ratio, 3),
        fl!(I18N, "ghz")
    )
}

fn format_current_gfxclk(value: Option<u64>) -> String {
    if let Some(v) = value {
        // if the APU/GPU dose not acually support current_gfxclk,
        // the value will be `u16::MAX (65535)`
        if v >= u16::MAX as u64 || v == 0 {
            fl!(I18N, "missing-stat")
        } else {
            format_clockspeed(Some(v), 1.0)
        }
    } else {
        fl!(I18N, "missing-stat")
    }
}

pub fn throttling_text(stats: &DeviceStats) -> String {
    match &stats.throttle_info {
        Some(throttle_info) => {
            if throttle_info.is_empty() {
                fl!(I18N, "no-throttling")
            } else {
                let type_text: Vec<String> = throttle_info
                    .iter()
                    .map(|(throttle_type, details)| {
                        let mut out = throttle_type.to_string();
                        if !details.is_empty() {
                            let _ = write!(out, "({})", details.join(", "));
                        }
                        out
                    })
                    .collect();

                type_text.join(", ")
            }
        }
        None => {
            fl!(I18N, "unknown-throttling")
        }
    }
}

pub fn temperature_text(stats: &DeviceStats) -> Option<String> {
    let mut temperatures: Vec<String> = stats
        .temps
        .iter()
        .filter_map(|(label, temp)| {
            temp.value
                .current
                .map(|current| format!("{label}: {}°C", Mono::float(current, 0)))
        })
        .collect();
    temperatures.sort_unstable();
    if temperatures.is_empty() {
        None
    } else {
        Some(temperatures.join(", "))
    }
}

pub fn fan_speed_text(stats: &DeviceStats) -> Option<String> {
    let fan_percent = stats
        .fan
        .pwm_current
        .map(|current_pwm| ((current_pwm as f64 / u8::MAX as f64) * 100.0).round());
    if let Some(current_rpm) = stats.fan.speed_current {
        let text = match fan_percent {
            Some(percent) => format!(
                "<b>{} RPM ({}%)</b>",
                Mono::uint(current_rpm),
                Mono::uint(percent as u32)
            ),
            None => format!("<b>{} RPM</b>", Mono::uint(current_rpm)),
        };
        Some(text)
    } else {
        fan_percent.map(|percent| format!("<b>{}%</b>", Mono::uint(percent as u32)))
    }
}
