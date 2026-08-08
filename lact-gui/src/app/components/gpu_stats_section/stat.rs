use crate::{
    I18N,
    app::utils::formatting::{self, Mono},
};
use i18n_embed_fl::fl;
use lact_schema::{DeviceStats, PowerStats};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GpuStatDisplay {
    Text,
    LevelBar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GpuStat {
    DeviceName,
    Throttling,
    GpuClockTarget,
    GpuVoltage,
    Temperature,
    GpuClock,
    VramClock,
    GpuUsage,
    VramUsage,
    GttUsage,
    PowerUsage,
    FanSpeed,
    #[serde(other)]
    Unknown,
}

impl GpuStat {
    pub const ALL: &[Self] = &[
        Self::DeviceName,
        Self::Throttling,
        Self::GpuClockTarget,
        Self::GpuVoltage,
        Self::Temperature,
        Self::GpuClock,
        Self::VramClock,
        Self::GpuUsage,
        Self::VramUsage,
        Self::GttUsage,
        Self::PowerUsage,
        Self::FanSpeed,
    ];

    pub fn title(self) -> String {
        match self {
            Self::DeviceName => fl!(I18N, "device-name"),
            Self::Throttling => fl!(I18N, "throttling"),
            Self::GpuClockTarget => fl!(I18N, "gpu-clock-target"),
            Self::GpuVoltage => fl!(I18N, "gpu-voltage"),
            Self::Temperature => fl!(I18N, "gpu-temp"),
            Self::GpuClock => fl!(I18N, "gpu-clock"),
            Self::VramClock => fl!(I18N, "vram-clock"),
            Self::GpuUsage => fl!(I18N, "gpu-usage"),
            Self::VramUsage => fl!(I18N, "vram-usage"),
            Self::GttUsage => fl!(I18N, "gtt-usage"),
            Self::PowerUsage => fl!(I18N, "power-usage"),
            Self::FanSpeed => fl!(I18N, "fan-speed"),
            Self::Unknown => String::new(),
        }
    }

    pub fn supported_displays(self) -> &'static [GpuStatDisplay] {
        use GpuStatDisplay::{LevelBar, Text};

        match self {
            Self::DeviceName
            | Self::Throttling
            | Self::GpuClockTarget
            | Self::GpuVoltage
            | Self::Temperature => &[Text],
            Self::GpuClock
            | Self::VramClock
            | Self::GpuUsage
            | Self::VramUsage
            | Self::GttUsage
            | Self::PowerUsage
            | Self::FanSpeed => &[Text, LevelBar],
            Self::Unknown => &[],
        }
    }

    pub fn default_display(self) -> GpuStatDisplay {
        match self {
            Self::DeviceName
            | Self::Throttling
            | Self::GpuClockTarget
            | Self::GpuVoltage
            | Self::Temperature
            | Self::Unknown => GpuStatDisplay::Text,
            Self::GpuClock
            | Self::VramClock
            | Self::GpuUsage
            | Self::VramUsage
            | Self::GttUsage
            | Self::PowerUsage
            | Self::FanSpeed => GpuStatDisplay::LevelBar,
        }
    }

    pub fn has_data_for(self, stats: &DeviceStats) -> bool {
        match self {
            Self::DeviceName | Self::Throttling | Self::Temperature | Self::VramUsage => true,
            Self::GpuClockTarget => stats.clockspeed.target_gpu_clockspeed.is_some(),
            Self::GpuVoltage => stats.voltage.gpu.is_some(),
            Self::GpuClock => stats.clockspeed.gpu_clockspeed.is_some(),
            Self::VramClock => stats.clockspeed.vram_clockspeed.is_some(),
            Self::GpuUsage => stats.busy_percent.is_some(),
            Self::GttUsage => stats.vram.gtt_used.is_some(),
            Self::PowerUsage => stats.power.average.is_some() || stats.power.current.is_some(),
            Self::FanSpeed => stats.fan.pwm_current.is_some() || stats.fan.speed_current.is_some(),
            Self::Unknown => false,
        }
    }

    pub fn name(self, ctx: &StatsContext) -> String {
        if self == Self::GpuClock
            && ctx.stats.clockspeed.gpu_clockspeed.is_some()
            && ctx.stats.clockspeed.target_gpu_clockspeed.is_some()
        {
            fl!(I18N, "gpu-clock-avg")
        } else {
            self.title()
        }
    }

    pub fn text(self, ctx: &StatsContext) -> String {
        match self {
            Self::DeviceName => ctx.gpu_model.clone(),
            Self::Throttling => formatting::fmt_throttling_text(&ctx.stats),
            Self::GpuClockTarget => {
                format_current_gfxclk(ctx.stats.clockspeed.target_gpu_clockspeed)
            }
            Self::GpuVoltage => format!(
                "{} V",
                Mono::float(ctx.stats.voltage.gpu.unwrap_or(0) as f64 / 1000f64, 3)
            ),
            Self::Temperature => {
                let (primary_temperatures, _) = formatting::fmt_temperature_text(&ctx.stats);
                if primary_temperatures.is_empty() {
                    "N/A".to_owned()
                } else {
                    primary_temperatures.join(", ")
                }
            }
            Self::GpuClock => formatting::fmt_clockspeed(ctx.stats.clockspeed.gpu_clockspeed, 1.0),
            Self::VramClock => formatting::fmt_clockspeed(
                ctx.stats.clockspeed.vram_clockspeed,
                ctx.vram_clock_ratio,
            ),
            Self::GpuUsage => format!("{}%", Mono::uint(ctx.stats.busy_percent.unwrap_or(0))),
            Self::VramUsage => formatting::fmt_human_bytes(
                ctx.stats.vram.used.unwrap_or(0),
                Some(formatting::ByteUnit::Gibibyte),
            ),
            Self::GttUsage => formatting::fmt_human_bytes(
                ctx.stats.vram.gtt_used.unwrap_or(0),
                Some(formatting::ByteUnit::Gibibyte),
            ),
            Self::PowerUsage => {
                let PowerStats {
                    average: power_average,
                    current: power_current,
                    ..
                } = ctx.stats.power;

                let power_current = power_current
                    .filter(|value| *value != 0.0)
                    .or(power_average);

                format!(
                    "{} {}",
                    Mono::float(power_current.unwrap_or(0.0), 1),
                    fl!(I18N, "watt")
                )
            }
            Self::FanSpeed => formatting::fmt_fan_speed(&ctx.stats, true)
                .unwrap_or_else(|| fl!(I18N, "missing-stat")),
            Self::Unknown => String::new(),
        }
    }

    pub fn level(self, ctx: &StatsContext) -> f64 {
        match self {
            Self::GpuClock => match (
                &ctx.stats.clockspeed.gpu_clockspeed,
                ctx.max_gpu_clock,
                ctx.min_gpu_clock,
            ) {
                (Some(cur), Some(max), Some(min)) if max > min => {
                    (cur.saturating_sub(min) as f64) / (max.saturating_sub(min) as f64)
                }
                _ => 0.0,
            },
            Self::VramClock => match (
                &ctx.stats.clockspeed.vram_clockspeed,
                ctx.max_vram_clock,
                ctx.min_vram_clock,
            ) {
                (Some(cur), Some(max), Some(min)) if max > min => {
                    (cur.saturating_sub(min) as f64) / (max.saturating_sub(min) as f64)
                }
                _ => 0.0,
            },
            Self::GpuUsage => ctx.stats.busy_percent.unwrap_or(0) as f64 / 100.0,
            Self::VramUsage => ctx
                .stats
                .vram
                .used
                .zip(ctx.stats.vram.total)
                .map(|(used, total)| used as f64 / total as f64)
                .unwrap_or(0.0),
            Self::GttUsage => ctx
                .stats
                .vram
                .gtt_used
                .zip(ctx.stats.vram.gtt_total_usable)
                .map(|(used, total)| used as f64 / total as f64)
                .unwrap_or(0.0),
            Self::PowerUsage => {
                let PowerStats {
                    average: power_average,
                    current: power_current,
                    cap_current: power_cap_current,
                    ..
                } = ctx.stats.power;

                let power_current = power_current
                    .filter(|value| *value != 0.0)
                    .or(power_average);
                let power_cap_current = power_cap_current.filter(|value| *value != 0.0);

                power_current
                    .zip(power_cap_current)
                    .map(|(current, cap)| current / cap)
                    .unwrap_or(0.0)
            }
            Self::FanSpeed => ctx
                .stats
                .fan
                .pwm_current
                .map(|pwm| pwm as f64 / u8::MAX as f64)
                .unwrap_or(0.0),
            Self::DeviceName
            | Self::Throttling
            | Self::GpuClockTarget
            | Self::GpuVoltage
            | Self::Temperature
            | Self::Unknown => 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StatsContext {
    pub stats: Arc<DeviceStats>,
    pub vram_clock_ratio: f64,
    pub gpu_model: String,
    pub min_gpu_clock: Option<u64>,
    pub max_gpu_clock: Option<u64>,
    pub min_vram_clock: Option<u64>,
    pub max_vram_clock: Option<u64>,
}

impl Default for StatsContext {
    fn default() -> Self {
        Self {
            stats: Arc::new(DeviceStats::default()),
            vram_clock_ratio: 1.0,
            gpu_model: String::new(),
            min_gpu_clock: None,
            max_gpu_clock: None,
            min_vram_clock: None,
            max_vram_clock: None,
        }
    }
}

fn format_current_gfxclk(value: Option<u64>) -> String {
    if let Some(v) = value {
        // Unsupported current_gfxclk reports u16::MAX on some GPUs.
        if v >= u16::MAX as u64 || v == 0 {
            fl!(I18N, "missing-stat")
        } else {
            formatting::fmt_clockspeed(Some(v), 1.0)
        }
    } else {
        fl!(I18N, "missing-stat")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_covers_every_known_stat() {
        assert_eq!(GpuStat::ALL.len(), 12);
        assert!(GpuStat::ALL.iter().all(|stat| {
            !stat.supported_displays().is_empty()
                && stat.supported_displays().contains(&stat.default_display())
        }));
        assert_eq!(
            GpuStat::Temperature.supported_displays(),
            &[GpuStatDisplay::Text]
        );
        assert_eq!(
            GpuStat::GpuClock.supported_displays(),
            &[GpuStatDisplay::Text, GpuStatDisplay::LevelBar]
        );
    }

    #[test]
    fn availability_matches_reported_fields() {
        let empty = DeviceStats::default();
        for stat in [
            GpuStat::DeviceName,
            GpuStat::Throttling,
            GpuStat::Temperature,
            GpuStat::VramUsage,
        ] {
            assert!(stat.has_data_for(&empty));
        }
        assert!(!GpuStat::GpuClock.has_data_for(&empty));
        assert!(!GpuStat::PowerUsage.has_data_for(&empty));

        let mut reported = DeviceStats::default();
        reported.clockspeed.gpu_clockspeed = Some(1200);
        reported.power.average = Some(100.0);
        assert!(GpuStat::GpuClock.has_data_for(&reported));
        assert!(GpuStat::PowerUsage.has_data_for(&reported));
    }

    #[test]
    fn levels_preserve_existing_calculations() {
        let mut stats = DeviceStats::default();
        stats.clockspeed.gpu_clockspeed = Some(1500);
        stats.vram.used = Some(3);
        stats.vram.total = Some(4);
        stats.power.current = Some(90.0);
        stats.power.cap_current = Some(120.0);
        let ctx = StatsContext {
            stats: Arc::new(stats),
            min_gpu_clock: Some(500),
            max_gpu_clock: Some(2500),
            ..StatsContext::default()
        };

        assert_eq!(GpuStat::GpuClock.level(&ctx), 0.5);
        assert_eq!(GpuStat::VramUsage.level(&ctx), 0.75);
        assert_eq!(GpuStat::PowerUsage.level(&ctx), 0.75);
    }
}
