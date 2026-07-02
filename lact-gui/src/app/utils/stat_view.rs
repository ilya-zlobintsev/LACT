use i18n_embed_fl::fl;
use lact_schema::DeviceStats;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::LazyLock};

use crate::I18N;

use super::formatting::{self, Mono};

pub(crate) type StatConfigMap = HashMap<StatType, StatConfig>;
type ValueFn = fn(&StatType, &StatContext<'_>) -> Option<f64>;
type FormatFn = fn(&StatType, Option<f64>, &StatContext<'_>) -> String;
type VisibleFn = fn(&StatType, Option<f64>, &StatContext<'_>) -> bool;
type LevelFn = fn(&StatType, Option<f64>, &StatContext<'_>) -> f64;

#[derive(Debug, Clone)]
pub(crate) struct StatConfig {
    pub label: String,
    pub unit_label: &'static str,
    pub show_peak: bool,
    pub graphable: bool,
    value: ValueFn,
    format: Option<FormatFn>,
    visible: Option<VisibleFn>,
    level: Option<LevelFn>,
}

impl StatConfig {
    pub(crate) fn value(&self, stat_type: &StatType, context: &StatContext<'_>) -> Option<f64> {
        (self.value)(stat_type, context)
    }

    pub(crate) fn format(&self, stat_type: &StatType, context: &StatContext<'_>) -> Option<String> {
        let value = self.value(stat_type, context);
        self.format.map(|format| format(stat_type, value, context))
    }

    pub(crate) fn visible(&self, stat_type: &StatType, context: &StatContext<'_>) -> Option<bool> {
        let value = self.value(stat_type, context);
        self.visible
            .map(|visible| visible(stat_type, value, context))
    }

    pub(crate) fn level(&self, stat_type: &StatType, context: &StatContext<'_>) -> Option<f64> {
        let value = self.value(stat_type, context);
        self.level.map(|level| level(stat_type, value, context))
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct StatContext<'a> {
    pub stats: &'a DeviceStats,
    pub vram_clock_ratio: f64,
    pub max_gpu_clock: Option<u64>,
    pub max_vram_clock: Option<u64>,
    pub min_gpu_clock: Option<u64>,
    pub min_vram_clock: Option<u64>,
}

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Serialize, Deserialize)]
pub enum StatType {
    GpuClock,
    GpuTargetClock,
    GpuUsage,
    Temperature(String),
    FanRpm,
    FanPwm,
    PowerCurrent,
    PowerAverage,
    PowerCap,
    Power(String),
    VramClock,
    VramSize,
    VramUsed,
    GttSize,
    GttUsed,
    GpuVoltage,
    Clockspeed(String),
    Voltage(String),
    Throttling,
    Temperatures,
    VramUsage,
    GttUsage,
    PowerUsage,
    FanSpeed,
}

impl StatType {
    pub(crate) fn precision(&self) -> usize {
        use StatType::*;
        match self {
            GpuClock | GpuTargetClock | VramClock | Clockspeed(_) => 0,
            FanPwm => 1,
            FanRpm => 0,
            PowerCurrent | PowerAverage | Power(_) => 1,
            PowerCap => 0,
            Temperature(_) => 1,
            GpuUsage | VramSize | VramUsed | GttSize | GttUsed => 0,
            GpuVoltage | Voltage(_) => 0,
            Throttling | Temperatures | VramUsage | GttUsage | PowerUsage | FanSpeed => 0,
        }
    }
}

pub(crate) fn build_stat_config_map(context: &StatContext<'_>) -> StatConfigMap {
    let mut configs = static_stat_configs().clone();

    for name in context.stats.temps.keys() {
        let stat_type = StatType::Temperature(name.clone());
        let config = StatConfig {
            label: format!("{} ({name})", fl!(I18N, "gpu-temp")),
            unit_label: "℃",
            show_peak: true,
            graphable: true,
            value: |stat_type, context| {
                let StatType::Temperature(name) = stat_type else {
                    return None;
                };

                context
                    .stats
                    .temps
                    .get(name)
                    .and_then(|temperature| temperature.value.current)
                    .map(Into::into)
            },
            format: None,
            visible: None,
            level: None,
        };
        configs.insert(stat_type, config);
    }

    for name in context.stats.voltage.sensors.keys() {
        let stat_type = StatType::Voltage(name.clone());
        let config = StatConfig {
            label: format!("{} ({name})", fl!(I18N, "voltage")),
            unit_label: "mV",
            show_peak: true,
            graphable: true,
            value: |stat_type, context| {
                let StatType::Voltage(name) = stat_type else {
                    return None;
                };

                context
                    .stats
                    .voltage
                    .sensors
                    .get(name)
                    .map(|val| *val as f64)
            },
            format: None,
            visible: None,
            level: None,
        };
        configs.insert(stat_type, config);
    }

    for name in context.stats.clockspeed.sensors.keys() {
        let stat_type = StatType::Clockspeed(name.clone());
        let config = StatConfig {
            label: format!("Clockspeed ({name})"),
            unit_label: "MHz",
            show_peak: true,
            graphable: true,
            value: |stat_type, context| {
                let StatType::Clockspeed(name) = stat_type else {
                    return None;
                };

                context
                    .stats
                    .clockspeed
                    .sensors
                    .get(name)
                    .map(|val| *val as f64)
            },
            format: None,
            visible: None,
            level: None,
        };
        configs.insert(stat_type, config);
    }

    for name in context.stats.power.sensors.keys() {
        let stat_type = StatType::Power(name.clone());
        let config = StatConfig {
            label: format!("{} ({name})", fl!(I18N, "power-usage")),
            unit_label: "W",
            show_peak: true,
            graphable: true,
            value: |stat_type, context| {
                let StatType::Power(name) = stat_type else {
                    return None;
                };

                context.stats.power.sensors.get(name).copied()
            },
            format: None,
            visible: None,
            level: None,
        };
        configs.insert(stat_type, config);
    }

    configs
}

pub(crate) fn static_stat_configs() -> &'static StatConfigMap {
    static CONFIGS: LazyLock<StatConfigMap> = LazyLock::new(|| {
        HashMap::from([
            (
                StatType::GpuClock,
                StatConfig {
                    label: fl!(I18N, "gpu-clock"),
                    unit_label: "MHz",
                    show_peak: true,
                    graphable: true,
                    value: |_, context| {
                        context
                            .stats
                            .clockspeed
                            .gpu_clockspeed
                            .map(|val| val as f64)
                    },
                    format: Some(|_, value, _| format_mhz_value(value)),
                    visible: Some(|_, value, _| value.is_some()),
                    level: Some(|_, value, context| {
                        clock_level(
                            value,
                            context.min_gpu_clock.map(|value| value as f64),
                            context.max_gpu_clock.map(|value| value as f64),
                        )
                    }),
                },
            ),
            (
                StatType::GpuTargetClock,
                StatConfig {
                    label: fl!(I18N, "gpu-clock-target"),
                    unit_label: "MHz",
                    show_peak: true,
                    graphable: true,
                    value: |_, context| {
                        context
                            .stats
                            .clockspeed
                            .target_gpu_clockspeed
                            .map(|val| val as f64)
                    },
                    format: Some(|_, value, _| format_current_gfxclk(value)),
                    visible: Some(|_, value, _| value.is_some()),
                    level: None,
                },
            ),
            (
                StatType::GpuUsage,
                StatConfig {
                    label: fl!(I18N, "gpu-usage"),
                    unit_label: "%",
                    show_peak: true,
                    graphable: true,
                    value: |_, context| context.stats.busy_percent.map(|val| val as f64),
                    format: Some(|_, value, _| {
                        format!("{}%", Mono::uint(value.unwrap_or(0.0) as u64))
                    }),
                    visible: Some(|_, value, _| value.is_some()),
                    level: Some(|_, value, _| value.unwrap_or(0.0) / 100.0),
                },
            ),
            (
                StatType::FanRpm,
                StatConfig {
                    label: "Fan RPM".into(),
                    unit_label: "RPM",
                    show_peak: true,
                    graphable: true,
                    value: |_, context| context.stats.fan.speed_current.map(|val| val as f64),
                    format: None,
                    visible: None,
                    level: None,
                },
            ),
            (
                StatType::FanPwm,
                StatConfig {
                    label: "Fan".into(),
                    unit_label: "%",
                    show_peak: true,
                    graphable: true,
                    value: |_, context| {
                        context
                            .stats
                            .fan
                            .pwm_current
                            .map(|val| (val as f64) / u8::MAX as f64 * 100.0)
                    },
                    format: None,
                    visible: None,
                    level: None,
                },
            ),
            (
                StatType::PowerCurrent,
                StatConfig {
                    label: "Power Draw".into(),
                    unit_label: "W",
                    show_peak: true,
                    graphable: true,
                    value: |_, context| context.stats.power.current,
                    format: None,
                    visible: None,
                    level: None,
                },
            ),
            (
                StatType::PowerAverage,
                StatConfig {
                    label: "Power Draw (Avg)".into(),
                    unit_label: "W",
                    show_peak: true,
                    graphable: true,
                    value: |_, context| context.stats.power.average,
                    format: None,
                    visible: None,
                    level: None,
                },
            ),
            (
                StatType::PowerCap,
                StatConfig {
                    label: fl!(I18N, "power-cap"),
                    unit_label: "W",
                    show_peak: false,
                    graphable: true,
                    value: |_, context| context.stats.power.cap_current,
                    format: None,
                    visible: None,
                    level: None,
                },
            ),
            (
                StatType::VramClock,
                StatConfig {
                    label: fl!(I18N, "vram-clock"),
                    unit_label: "MHz",
                    show_peak: true,
                    graphable: true,
                    value: |_, context| {
                        context
                            .stats
                            .clockspeed
                            .vram_clockspeed
                            .map(|val| val as f64 * context.vram_clock_ratio)
                    },
                    format: Some(|_, value, _| format_mhz_value(value)),
                    visible: Some(|_, value, _| value.is_some()),
                    level: Some(|_, value, context| {
                        clock_level(
                            value,
                            context
                                .min_vram_clock
                                .map(|value| value as f64 * context.vram_clock_ratio),
                            context
                                .max_vram_clock
                                .map(|value| value as f64 * context.vram_clock_ratio),
                        )
                    }),
                },
            ),
            (
                StatType::VramSize,
                StatConfig {
                    label: "VRAM Size".into(),
                    unit_label: "MiB",
                    show_peak: false,
                    graphable: true,
                    value: |_, context| {
                        context
                            .stats
                            .vram
                            .total
                            .map(|val| (val / 1024 / 1024) as f64)
                    },
                    format: None,
                    visible: None,
                    level: None,
                },
            ),
            (
                StatType::VramUsed,
                StatConfig {
                    label: "VRAM Used".into(),
                    unit_label: "MiB",
                    show_peak: true,
                    graphable: true,
                    value: |_, context| {
                        context
                            .stats
                            .vram
                            .used
                            .map(|val| (val / 1024 / 1024) as f64)
                    },
                    format: None,
                    visible: None,
                    level: None,
                },
            ),
            (
                StatType::GttSize,
                StatConfig {
                    label: "GTT Size".into(),
                    unit_label: "MiB",
                    show_peak: false,
                    graphable: true,
                    value: |_, context| {
                        context
                            .stats
                            .vram
                            .gtt_total_usable
                            .map(|val| (val / 1024 / 1024) as f64)
                    },
                    format: None,
                    visible: None,
                    level: None,
                },
            ),
            (
                StatType::GttUsed,
                StatConfig {
                    label: "GTT Used".into(),
                    unit_label: "MiB",
                    show_peak: true,
                    graphable: true,
                    value: |_, context| {
                        context
                            .stats
                            .vram
                            .gtt_used
                            .map(|val| (val / 1024 / 1024) as f64)
                    },
                    format: None,
                    visible: None,
                    level: None,
                },
            ),
            (
                StatType::GpuVoltage,
                StatConfig {
                    label: fl!(I18N, "gpu-voltage"),
                    unit_label: "mV",
                    show_peak: true,
                    graphable: true,
                    value: |_, context| context.stats.voltage.gpu.map(|val| val as f64),
                    format: Some(|_, value, _| {
                        format!("{} V", Mono::float(value.unwrap_or(0.0) / 1000f64, 3))
                    }),
                    visible: Some(|_, value, _| value.is_some()),
                    level: None,
                },
            ),
            (
                StatType::Throttling,
                StatConfig {
                    label: fl!(I18N, "throttling"),
                    unit_label: "",
                    show_peak: false,
                    graphable: false,
                    value: |_, _| None,
                    format: Some(|_, _value, context| {
                        formatting::fmt_throttling_text(context.stats)
                    }),
                    visible: Some(|_, _value, _| true),
                    level: None,
                },
            ),
            (
                StatType::Temperatures,
                StatConfig {
                    label: fl!(I18N, "gpu-temp"),
                    unit_label: "℃",
                    show_peak: false,
                    graphable: false,
                    value: |_, _| None,
                    format: Some(|_, _value, context| {
                        let (primary, _) = formatting::fmt_temperature_text(context.stats);
                        if primary.is_empty() {
                            fl!(I18N, "missing-stat")
                        } else {
                            primary.join(", ")
                        }
                    }),
                    visible: Some(|_, _value, _| true),
                    level: None,
                },
            ),
            (
                StatType::VramUsage,
                StatConfig {
                    label: fl!(I18N, "vram-usage"),
                    unit_label: "",
                    show_peak: false,
                    graphable: false,
                    value: |_, context| {
                        context
                            .stats
                            .vram
                            .used
                            .zip(context.stats.vram.total)
                            .map(|(used, total)| used as f64 / total as f64)
                    },
                    format: Some(|_, _value, context| {
                        formatting::fmt_human_bytes(
                            context.stats.vram.used.unwrap_or(0),
                            Some(formatting::ByteUnit::Gibibyte),
                        )
                    }),
                    visible: Some(|_, _value, _| true),
                    level: Some(|_, value, _| value.unwrap_or(0.0)),
                },
            ),
            (
                StatType::PowerUsage,
                StatConfig {
                    label: fl!(I18N, "power-usage"),
                    unit_label: "W",
                    show_peak: false,
                    graphable: false,
                    value: |_, context| power_usage_value(context.stats),
                    format: Some(|_, value, _| {
                        format!(
                            "{} {}",
                            Mono::float(value.unwrap_or(0.0), 1),
                            fl!(I18N, "watt")
                        )
                    }),
                    visible: Some(|_, value, _| value.is_some()),
                    level: Some(|_, value, context| {
                        value
                            .zip(
                                context
                                    .stats
                                    .power
                                    .cap_current
                                    .filter(|value| *value != 0.0),
                            )
                            .map(|(current, cap)| current / cap)
                            .unwrap_or(0.0)
                    }),
                },
            ),
            (
                StatType::GttUsage,
                StatConfig {
                    label: fl!(I18N, "gtt-usage"),
                    unit_label: "",
                    show_peak: false,
                    graphable: false,
                    value: |_, context| {
                        context
                            .stats
                            .vram
                            .gtt_used
                            .zip(context.stats.vram.gtt_total_usable)
                            .map(|(used, total)| used as f64 / total as f64)
                    },
                    format: Some(|_, _value, context| {
                        formatting::fmt_human_bytes(
                            context.stats.vram.gtt_used.unwrap_or(0),
                            Some(formatting::ByteUnit::Gibibyte),
                        )
                    }),
                    visible: Some(|_, value, _| value.is_some()),
                    level: Some(|_, value, _| value.unwrap_or(0.0)),
                },
            ),
            (
                StatType::FanSpeed,
                StatConfig {
                    label: fl!(I18N, "fan-speed"),
                    unit_label: "",
                    show_peak: false,
                    graphable: false,
                    value: |_, context| {
                        context
                            .stats
                            .fan
                            .pwm_current
                            .map(|pwm| pwm as f64 / u8::MAX as f64)
                    },
                    format: Some(|_, _value, context| {
                        formatting::fmt_fan_speed(context.stats, true)
                            .unwrap_or_else(|| fl!(I18N, "missing-stat"))
                    }),
                    visible: Some(|_, _value, context| {
                        context.stats.fan.pwm_current.is_some()
                            || context.stats.fan.speed_current.is_some()
                    }),
                    level: Some(|_, value, _| value.unwrap_or(0.0)),
                },
            ),
        ])
    });

    &CONFIGS
}
fn power_usage_value(stats: &DeviceStats) -> Option<f64> {
    stats
        .power
        .current
        .filter(|value| *value != 0.0)
        .or(stats.power.average)
}

fn clock_level(current: Option<f64>, min: Option<f64>, max: Option<f64>) -> f64 {
    match (current, max, min) {
        (Some(cur), Some(max), Some(min)) if max > min => (cur - min).max(0.0) / (max - min),
        _ => 0.0,
    }
}

fn format_mhz_value(value: Option<f64>) -> String {
    formatting::fmt_clockspeed(value, 1.0)
}

fn format_current_gfxclk(value: Option<f64>) -> String {
    if let Some(value) = value {
        // If the APU/GPU does not actually support current_gfxclk, the value will be u16::MAX.
        if value >= u16::MAX as f64 || value == 0.0 {
            fl!(I18N, "missing-stat")
        } else {
            format_mhz_value(Some(value))
        }
    } else {
        fl!(I18N, "missing-stat")
    }
}
