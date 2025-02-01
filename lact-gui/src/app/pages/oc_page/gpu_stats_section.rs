use crate::app::{info_row::InfoRow, page_section::PageSection, pages::PageUpdate};
use gtk::prelude::{ActionableExt, BoxExt, ButtonExt, OrientableExt, WidgetExt};
use lact_schema::{DeviceStats, PowerStats};
use relm4::{ComponentParts, ComponentSender};
use std::{fmt::Write, sync::Arc};

pub struct GpuStatsSection {
    stats: Arc<DeviceStats>,
    vram_clock_ratio: f64,
}

#[relm4::component(pub)]
impl relm4::SimpleComponent for GpuStatsSection {
    type Init = ();
    type Input = PageUpdate;
    type Output = ();

    view! {
        PageSection::new("Statistics") {
            set_spacing: 10,

            append = &gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 5,

                gtk::Label {
                    set_label: "VRAM Usage:",
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
                            "{}/{} MiB",
                            model.stats.vram.used.unwrap_or(0) / 1024 / 1024,
                            model.stats.vram.total.unwrap_or(0) / 1024 / 1024,
                        ),
                    }
                },
            },

            append = &gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 10,
                set_homogeneous: true,

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_hexpand: true,
                    set_spacing: 5,

                    InfoRow {
                        set_name: "GPU Core Clock (Average):",
                        #[watch]
                        set_value: format_clockspeed(model.stats.clockspeed.gpu_clockspeed, 1.0),
                    },

                    InfoRow {
                        set_name: "GPU Core Clock (Target):",
                        #[watch]
                        set_value: format_current_gfxclk(model.stats.clockspeed.current_gfxclk),
                    },

                    InfoRow {
                        set_name: "GPU Voltage:",
                        #[watch]
                        set_value: format!("{:.3} V", model.stats.voltage.gpu.unwrap_or(0) as f64 / 1000f64),
                    },

                    InfoRow {
                        set_name: "GPU Temperature (hotspot):",
                        #[watch]
                        set_value: {
                            let temperature = if model.stats.temps.len() == 1 {
                                model.stats.temps.values().next().unwrap().current
                            } else {
                                model.stats
                                    .temps
                                    .get("junction")
                                    .or_else(|| model.stats.temps.get("edge"))
                                    .and_then(|temp| temp.current)
                            }
                            .unwrap_or(0.0);
                            format!("{temperature}°C")
                        },
                    },
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_hexpand: true,
                    set_spacing: 5,

                    InfoRow {
                        set_name: "GPU Memory Clock:",
                        #[watch]
                        set_value: format_clockspeed(
                            model.stats.clockspeed.vram_clockspeed,
                            model.vram_clock_ratio,
                        ),
                    },

                    InfoRow {
                        set_name: "GPU Usage:",
                        #[watch]
                        set_value: format!("{}%", model.stats.busy_percent.unwrap_or(0)),
                    },

                    InfoRow {
                        set_name: "Power Usage:",
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

                            format!(
                                "<b>{:.1}/{} W</b>",
                                power_current.unwrap_or(0.0),
                                power_cap_current.unwrap_or(0.0)
                            )
                        }
                    },

                    InfoRow {
                        set_name: "Throttling:",
                        #[watch]
                        set_value: {
                            match &model.stats.throttle_info {
                                Some(throttle_info) => {
                                    if throttle_info.is_empty() {
                                        "No".to_owned()
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
                                None => "Unknown".to_owned(),
                            }
                        }
                    }
                }
            },

            append = &gtk::Button {
                set_label: "Show historical charts",
                set_action_name: Some("app.show-graphs-window"),
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
        };

        let widgets = view_output!();

        ComponentParts { widgets, model }
    }

    fn update(&mut self, msg: Self::Input, _sender: relm4::ComponentSender<Self>) {
        match msg {
            PageUpdate::Info(info) => {
                self.vram_clock_ratio = info.vram_clock_ratio();
            }
            PageUpdate::Stats(stats) => {
                self.stats = stats;
            }
        }
    }
}

fn format_clockspeed(value: Option<u64>, ratio: f64) -> String {
    format!("{:.3} GHz", value.unwrap_or(0) as f64 / 1000.0 * ratio)
}

fn format_current_gfxclk(value: Option<u64>) -> String {
    if let Some(v) = value {
        // if the APU/GPU dose not acually support current_gfxclk,
        // the value will be `u16::MAX (65535)`
        if v >= u16::MAX as u64 || v == 0 {
            "N/A".to_string()
        } else {
            format_clockspeed(Some(v), 1.0)
        }
    } else {
        "N/A".to_string()
    }
}
