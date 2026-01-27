use crate::app::msg::AppMsg;
use crate::app::APP_BROKER;
use crate::app::{
    ext::FlowBoxExt,
    formatting::{self, Mono},
    info_row::{InfoRow, InfoRowExt},
    info_row_level::InfoRowLevel,
    page_section::PageSection,
    pages::PageUpdate,
};
use crate::I18N;
use gtk::prelude::{ButtonExt, Cast, FlowBoxChildExt, OrientableExt, WidgetExt};
use i18n_embed_fl::fl;
use lact_schema::{DeviceStats, PowerStats};
use relm4::{ComponentParts, ComponentSender};
use std::sync::Arc;

pub struct GpuStatsSection {
    stats: Arc<DeviceStats>,
    vram_clock_ratio: f64,
    gpu_model: String,
    value_size_group: gtk::SizeGroup,
}

#[relm4::component(pub)]
impl relm4::SimpleComponent for GpuStatsSection {
    type Init = ();
    type Input = PageUpdate;
    type Output = ();

    view! {
        PageSection::new(&fl!(I18N, "stats-section")) {
            append_header = &gtk::Button {
                set_label: &fl!(I18N, "show-historical-charts"),
                connect_clicked => move |_| APP_BROKER.send(AppMsg::ShowGraphsWindow),
                set_halign: gtk::Align::End,
                set_hexpand: true,
            },

            append_child = &gtk::FlowBox {
                set_orientation: gtk::Orientation::Horizontal,
                set_column_spacing: 10,
                set_homogeneous: true,
                set_min_children_per_line: 2,
                set_max_children_per_line: 4,
                set_selection_mode: gtk::SelectionMode::None,

                append = &InfoRow {
                    set_name: fl!(I18N, "device-name"),
                    #[watch]
                    set_value: model.gpu_model.clone(),
                },

                append = &InfoRow {
                    set_name: fl!(I18N, "throttling"),
                    #[watch]
                    set_value: formatting::fmt_throttling_text(&model.stats),
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
                    set_value: formatting::fmt_clockspeed(
                        model.stats.clockspeed.gpu_clockspeed,
                        1.0,
                    ),
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
                    set_value: formatting::fmt_clockspeed(
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

                append_child = &InfoRowLevel {
                    set_name: fl!(I18N, "power-usage"),
                    #[watch]
                    set_value: {
                        let PowerStats {
                            average: power_average,
                            current: power_current,
                            ..
                        } = model.stats.power;

                        let power_current = power_current
                            .filter(|value| *value != 0.0)
                            .or(power_average);

                        format!(
                            "{} {}",
                            Mono::float(power_current.unwrap_or(0.0), 1),
                            fl!(I18N, "watt")
                        )
                    },
                    #[watch]
                    set_level_value: {
                        let PowerStats {
                            average: power_average,
                            current: power_current,
                            cap_current: power_cap_current,
                            ..
                        } = model.stats.power;

                        let power_current = power_current
                            .filter(|value| *value != 0.0)
                            .or(power_average);

                        power_current
                            .zip(power_cap_current)
                            .map(|(current, cap)| current / cap)
                            .unwrap_or(0.0)
                    },
                } -> power_usage_item: gtk::FlowBoxChild {
                    #[watch]
                    set_visible: model.stats.power.average.is_some() || model.stats.power.current.is_some(),
                },

                append_child = &InfoRowLevel {
                    set_name: fl!(I18N, "gpu-usage"),
                    #[watch]
                    set_value: format!("{}%", Mono::uint(model.stats.busy_percent.unwrap_or(0))),
                    #[watch]
                    set_level_value: model.stats.busy_percent.unwrap_or(0) as f64 / 100.0,
                } -> gpu_usage_item: gtk::FlowBoxChild {
                    #[watch]
                    set_visible: model.stats.busy_percent.is_some(),
                },


                append_child = &InfoRowLevel {
                    set_name: fl!(I18N, "vram-usage"),
                    #[watch]
                    set_value: formatting::fmt_human_bytes(
                        model.stats.vram.used.unwrap_or(0),
                        Some(formatting::ByteUnit::Gibibyte),
                    ),
                    #[watch]
                    set_level_value: model
                        .stats
                        .vram
                        .used
                        .zip(model.stats.vram.total)
                        .map(|(used, total)| used as f64 / total as f64)
                        .unwrap_or(0.0),
                } -> vram_usage_item: gtk::FlowBoxChild {},

                append = &InfoRow {
                    set_name: fl!(I18N, "gpu-temp"),
                    #[watch]
                    set_value: formatting::fmt_temperature_text(&model.stats)
                        .unwrap_or_else(|| "N/A".to_owned()),
                },

                append_child = &InfoRow {
                    set_name: fl!(I18N, "fan-speed"),
                    #[watch]
                    set_value: formatting::fmt_fan_speed(&model.stats)
                        .unwrap_or_else(|| fl!(I18N, "missing-stat")),
                } -> fan_speed_item: gtk::FlowBoxChild {
                    #[watch]
                    set_visible: model.stats.fan.pwm_current.is_some() || model.stats.fan.speed_current.is_some(),
                },
            },
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let value_size_group = gtk::SizeGroup::new(gtk::SizeGroupMode::Horizontal);

        let model = Self {
            stats: Arc::new(DeviceStats::default()),
            vram_clock_ratio: 1.0,
            gpu_model: String::new(),
            value_size_group,
        };

        let widgets = view_output!();

        widgets
            .power_usage_item
            .child()
            .unwrap()
            .downcast::<InfoRowLevel>()
            .unwrap()
            .set_value_size_group(&model.value_size_group);
        widgets
            .gpu_usage_item
            .child()
            .unwrap()
            .downcast::<InfoRowLevel>()
            .unwrap()
            .set_value_size_group(&model.value_size_group);
        widgets
            .vram_usage_item
            .child()
            .unwrap()
            .downcast::<InfoRowLevel>()
            .unwrap()
            .set_value_size_group(&model.value_size_group);

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

fn format_current_gfxclk(value: Option<u64>) -> String {
    if let Some(v) = value {
        // if the APU/GPU dose not actually support current_gfxclk,
        // the value will be `u16::MAX (65535)`
        if v >= u16::MAX as u64 || v == 0 {
            fl!(I18N, "missing-stat")
        } else {
            formatting::fmt_clockspeed(Some(v), 1.0)
        }
    } else {
        fl!(I18N, "missing-stat")
    }
}
