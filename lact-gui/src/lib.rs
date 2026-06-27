mod app;
mod config;

use std::{
    borrow::Cow,
    panic,
    sync::{LazyLock, atomic::AtomicBool, atomic::Ordering},
};

use anyhow::Context;
use app::{
    APP_BROKER, AppModel,
    msg::AppMsg,
    utils::formatting::{self, Mono},
};
use config::UiConfig;
use i18n_embed::fluent::{FluentLanguageLoader, fluent_language_loader};
use i18n_embed_fl::fl;
use lact_schema::{DeviceStats, args::GuiArgs, i18n};
use relm4::{
    RelmApp, SharedState,
    gtk::{glib, glib::MainContext},
};
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use tracing::metadata::LevelFilter;
use tracing_subscriber::EnvFilter;

static CONFIG: SharedState<UiConfig> = SharedState::new();
static PANICKED: AtomicBool = AtomicBool::new(false);

const GUI_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const APP_ID: &str = "io.github.ilya_zlobintsev.LACT";
pub const REPO_URL: &str = "https://github.com/ilya-zlobintsev/LACT";

pub(crate) static I18N: LazyLock<FluentLanguageLoader> = LazyLock::new(|| {
    i18n::loader(
        fluent_language_loader!(),
        &Localizations,
        cfg!(test).then(|| vec!["en-US".parse().unwrap()]),
    )
});

#[derive(RustEmbed)]
#[folder = "i18n"]
pub struct Localizations;

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
    DeviceName,
    Throttling,
    Temperatures,
    VramUsage,
    GttUsage,
    PowerUsage,
    FanSpeed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatView {
    Info,
    Level,
    Temperatures,
}

#[derive(Debug, Clone, Copy)]
pub struct StatContext<'a> {
    pub stats: &'a DeviceStats,
    pub gpu_model: &'a str,
    pub vram_clock_ratio: f64,
    pub max_gpu_clock: Option<u64>,
    pub max_vram_clock: Option<u64>,
    pub min_gpu_clock: Option<u64>,
    pub min_vram_clock: Option<u64>,
}

impl StatType {
    pub fn graph_values(stats: &DeviceStats, vram_clock_ratio: f64) -> Vec<(Self, f64)> {
        let mut values = Vec::new();

        for (name, temperature) in &stats.temps {
            if let Some(value) = temperature.value.current {
                values.push((Self::Temperature(name.to_owned()), value.into()));
            }
        }

        for (name, value) in &stats.voltage.sensors {
            values.push((Self::Voltage(name.clone()), *value as f64));
        }

        for (name, value) in &stats.clockspeed.sensors {
            values.push((Self::Clockspeed(name.clone()), *value as f64));
        }

        for (name, value) in &stats.power.sensors {
            values.push((Self::Power(name.clone()), *value));
        }

        for stat_type in [
            Self::GpuClock,
            Self::GpuTargetClock,
            Self::VramClock,
            Self::GpuVoltage,
            Self::PowerAverage,
            Self::PowerCurrent,
            Self::PowerCap,
            Self::FanPwm,
            Self::FanRpm,
            Self::GpuUsage,
            Self::VramSize,
            Self::VramUsed,
            Self::GttSize,
            Self::GttUsed,
        ] {
            if let Some(value) = stat_type.graph_value(stats, vram_clock_ratio) {
                values.push((stat_type, value));
            }
        }

        values
    }

    pub fn graph_value(&self, stats: &DeviceStats, vram_clock_ratio: f64) -> Option<f64> {
        match self {
            Self::GpuClock => stats.clockspeed.gpu_clockspeed.map(|val| val as f64),
            Self::GpuTargetClock => stats.clockspeed.target_gpu_clockspeed.map(|val| val as f64),
            Self::GpuUsage => stats.busy_percent.map(|val| val as f64),
            Self::Temperature(name) => stats
                .temps
                .get(name)
                .and_then(|temperature| temperature.value.current)
                .map(Into::into),
            Self::FanRpm => stats.fan.speed_current.map(|val| val as f64),
            Self::FanPwm => stats
                .fan
                .pwm_current
                .map(|val| (val as f64) / u8::MAX as f64 * 100.0),
            Self::PowerCurrent => stats.power.current,
            Self::PowerAverage => stats.power.average,
            Self::PowerCap => stats.power.cap_current,
            Self::Power(name) => stats.power.sensors.get(name).copied(),
            Self::VramClock => stats
                .clockspeed
                .vram_clockspeed
                .map(|val| val as f64 * vram_clock_ratio),
            Self::VramSize => stats.vram.total.map(|val| (val / 1024 / 1024) as f64),
            Self::VramUsed => stats.vram.used.map(|val| (val / 1024 / 1024) as f64),
            Self::GttSize => stats
                .vram
                .gtt_total_usable
                .map(|val| (val / 1024 / 1024) as f64),
            Self::GttUsed => stats.vram.gtt_used.map(|val| (val / 1024 / 1024) as f64),
            Self::GpuVoltage => stats.voltage.gpu.map(|val| val as f64),
            Self::Clockspeed(name) => stats.clockspeed.sensors.get(name).map(|val| *val as f64),
            Self::Voltage(name) => stats.voltage.sensors.get(name).map(|val| *val as f64),
            Self::DeviceName
            | Self::Throttling
            | Self::Temperatures
            | Self::VramUsage
            | Self::GttUsage
            | Self::PowerUsage
            | Self::FanSpeed => None,
        }
    }

    pub fn stat_view(&self) -> Option<StatView> {
        match self {
            Self::DeviceName | Self::Throttling | Self::GpuTargetClock | Self::GpuVoltage => {
                Some(StatView::Info)
            }
            Self::Temperatures => Some(StatView::Temperatures),
            Self::GpuClock
            | Self::VramClock
            | Self::GpuUsage
            | Self::VramUsage
            | Self::GttUsage
            | Self::PowerUsage
            | Self::FanSpeed => Some(StatView::Level),
            _ => None,
        }
    }

    pub fn gui_label(&self, context: &StatContext<'_>) -> String {
        match self {
            Self::DeviceName => fl!(I18N, "device-name"),
            Self::Throttling => fl!(I18N, "throttling"),
            Self::GpuTargetClock => fl!(I18N, "gpu-clock-target"),
            Self::GpuVoltage => fl!(I18N, "gpu-voltage"),
            Self::Temperatures => fl!(I18N, "gpu-temp"),
            Self::GpuClock => {
                if context.stats.clockspeed.gpu_clockspeed.is_some()
                    && context.stats.clockspeed.target_gpu_clockspeed.is_some()
                {
                    fl!(I18N, "gpu-clock-avg")
                } else {
                    fl!(I18N, "gpu-clock")
                }
            }
            Self::VramClock => fl!(I18N, "vram-clock"),
            Self::GpuUsage => fl!(I18N, "gpu-usage"),
            Self::VramUsage => fl!(I18N, "vram-usage"),
            Self::GttUsage => fl!(I18N, "gtt-usage"),
            Self::PowerUsage => fl!(I18N, "power-usage"),
            Self::FanSpeed => fl!(I18N, "fan-speed"),
            _ => self.graph_label().into_owned(),
        }
    }

    pub fn gui_value(&self, context: &StatContext<'_>) -> String {
        let stats = context.stats;
        match self {
            Self::DeviceName => context.gpu_model.to_owned(),
            Self::Throttling => formatting::fmt_throttling_text(stats),
            Self::GpuTargetClock => format_current_gfxclk(stats.clockspeed.target_gpu_clockspeed),
            Self::GpuVoltage => format!(
                "{} V",
                Mono::float(stats.voltage.gpu.unwrap_or(0) as f64 / 1000f64, 3)
            ),
            Self::Temperatures => {
                let (primary, _) = formatting::fmt_temperature_text(stats);
                if primary.is_empty() {
                    fl!(I18N, "missing-stat")
                } else {
                    primary.join(", ")
                }
            }
            Self::GpuClock => formatting::fmt_clockspeed(stats.clockspeed.gpu_clockspeed, 1.0),
            Self::VramClock => formatting::fmt_clockspeed(
                stats.clockspeed.vram_clockspeed,
                context.vram_clock_ratio,
            ),
            Self::GpuUsage => format!("{}%", Mono::uint(stats.busy_percent.unwrap_or(0))),
            Self::VramUsage => formatting::fmt_human_bytes(
                stats.vram.used.unwrap_or(0),
                Some(formatting::ByteUnit::Gibibyte),
            ),
            Self::GttUsage => formatting::fmt_human_bytes(
                stats.vram.gtt_used.unwrap_or(0),
                Some(formatting::ByteUnit::Gibibyte),
            ),
            Self::PowerUsage => format!(
                "{} {}",
                Mono::float(power_usage_value(stats).unwrap_or(0.0), 1),
                fl!(I18N, "watt")
            ),
            Self::FanSpeed => {
                formatting::fmt_fan_speed(stats, true).unwrap_or_else(|| fl!(I18N, "missing-stat"))
            }
            Self::Temperature(name) => stats
                .temps
                .get(name)
                .and_then(|temperature| temperature.value.current)
                .map(|current| format!("{}°C", Mono::float(current, 0)))
                .unwrap_or_else(|| fl!(I18N, "missing-stat")),
            Self::FanRpm => stats
                .fan
                .speed_current
                .map(|speed| format!("{} RPM", Mono::uint(speed)))
                .unwrap_or_else(|| fl!(I18N, "missing-stat")),
            Self::FanPwm => stats
                .fan
                .percent()
                .map(|percent| format!("{}%", Mono::uint(percent)))
                .unwrap_or_else(|| fl!(I18N, "missing-stat")),
            Self::PowerCurrent | Self::PowerAverage | Self::PowerCap | Self::Power(_) => self
                .graph_value(stats, context.vram_clock_ratio)
                .map(|value| format!("{} {}", Mono::float(value, 1), fl!(I18N, "watt")))
                .unwrap_or_else(|| fl!(I18N, "missing-stat")),
            Self::VramSize | Self::VramUsed | Self::GttSize | Self::GttUsed => {
                let value = match self {
                    Self::VramSize => stats.vram.total,
                    Self::VramUsed => stats.vram.used,
                    Self::GttSize => stats.vram.gtt_total_usable,
                    Self::GttUsed => stats.vram.gtt_used,
                    _ => unreachable!(),
                };
                value
                    .map(|value| formatting::fmt_human_bytes(value, None))
                    .unwrap_or_else(|| fl!(I18N, "missing-stat"))
            }
            Self::Clockspeed(_) => self
                .graph_value(stats, context.vram_clock_ratio)
                .map(|value| formatting::fmt_clockspeed(Some(value as u64), 1.0))
                .unwrap_or_else(|| fl!(I18N, "missing-stat")),
            Self::Voltage(_) => self
                .graph_value(stats, context.vram_clock_ratio)
                .map(|value| format!("{} mV", Mono::float(value, 0)))
                .unwrap_or_else(|| fl!(I18N, "missing-stat")),
        }
    }

    pub fn gui_visible(&self, context: &StatContext<'_>) -> bool {
        let stats = context.stats;
        match self {
            Self::DeviceName | Self::Throttling | Self::Temperatures | Self::VramUsage => true,
            Self::GttUsage => stats.vram.gtt_used.is_some(),
            Self::GpuTargetClock => stats.clockspeed.target_gpu_clockspeed.is_some(),
            Self::GpuVoltage => stats.voltage.gpu.is_some(),
            Self::GpuClock => stats.clockspeed.gpu_clockspeed.is_some(),
            Self::VramClock => stats.clockspeed.vram_clockspeed.is_some(),
            Self::GpuUsage => stats.busy_percent.is_some(),
            Self::PowerUsage => stats.power.average.is_some() || stats.power.current.is_some(),
            Self::FanSpeed => stats.fan.pwm_current.is_some() || stats.fan.speed_current.is_some(),
            _ => self.graph_value(stats, context.vram_clock_ratio).is_some(),
        }
    }

    pub fn gui_level(&self, context: &StatContext<'_>) -> Option<f64> {
        let stats = context.stats;
        match self {
            Self::GpuClock => Some(clock_level(
                stats.clockspeed.gpu_clockspeed,
                context.min_gpu_clock,
                context.max_gpu_clock,
            )),
            Self::VramClock => Some(clock_level(
                stats.clockspeed.vram_clockspeed,
                context.min_vram_clock,
                context.max_vram_clock,
            )),
            Self::GpuUsage => Some(stats.busy_percent.unwrap_or(0) as f64 / 100.0),
            Self::VramUsage => Some(
                stats
                    .vram
                    .used
                    .zip(stats.vram.total)
                    .map(|(used, total)| used as f64 / total as f64)
                    .unwrap_or(0.0),
            ),
            Self::GttUsage => Some(
                stats
                    .vram
                    .gtt_used
                    .zip(stats.vram.gtt_total_usable)
                    .map(|(used, total)| used as f64 / total as f64)
                    .unwrap_or(0.0),
            ),
            Self::PowerUsage => Some(
                power_usage_value(stats)
                    .zip(stats.power.cap_current.filter(|value| *value != 0.0))
                    .map(|(current, cap)| current / cap)
                    .unwrap_or(0.0),
            ),
            Self::FanSpeed => Some(
                stats
                    .fan
                    .pwm_current
                    .map(|pwm| pwm as f64 / u8::MAX as f64)
                    .unwrap_or(0.0),
            ),
            _ => None,
        }
    }

    pub fn temperature_values(&self, stats: &DeviceStats) -> Option<(Vec<String>, Vec<String>)> {
        matches!(self, Self::Temperatures).then(|| formatting::fmt_temperature_text(stats))
    }

    pub fn graph_label(&self) -> Cow<'static, str> {
        use StatType::*;
        match self {
            GpuClock => "Clockspeed (GPU)".into(),
            GpuTargetClock => "Clockspeed (GPU Target)".into(),
            GpuVoltage => "GPU Voltage".into(),
            VramClock => "Clockspeed (VRAM)".into(),
            VramSize => "VRAM Size".into(),
            VramUsed => "VRAM Used".into(),
            GttSize => "GTT Size".into(),
            GttUsed => "GTT Used".into(),
            GpuUsage => "GPU Usage".into(),
            Temperature(name) => format!("Temp ({name})").into(),
            Clockspeed(name) => format!("Clockspeed ({name})").into(),
            Voltage(name) => format!("Voltage ({name})").into(),
            Power(name) => format!("Power ({name})").into(),
            FanRpm => "Fan RPM".into(),
            FanPwm => "Fan".into(),
            PowerCurrent => "Power Draw".into(),
            PowerAverage => "Power Draw (Avg)".into(),
            PowerCap => "Power Cap".into(),
            DeviceName => "Device Name".into(),
            Throttling => "Throttling".into(),
            Temperatures => "Temperature".into(),
            VramUsage => "VRAM Usage".into(),
            GttUsage => "GTT Usage".into(),
            PowerUsage => "Power Usage".into(),
            FanSpeed => "Fan Speed".into(),
        }
    }

    pub fn unit_label(&self) -> &'static str {
        use StatType::*;
        match self {
            GpuClock | GpuTargetClock | VramClock | Clockspeed(_) => "MHz",
            VramSize | VramUsed | GttSize | GttUsed => "MiB",
            GpuVoltage | Voltage(_) => "mV",
            Temperature(_) | Temperatures => "℃",
            FanRpm => "RPM",
            FanPwm | GpuUsage => "%",
            PowerCurrent | PowerAverage | PowerCap | Power(_) | PowerUsage => "W",
            DeviceName | Throttling | VramUsage | GttUsage | FanSpeed => "",
        }
    }

    pub fn precision(&self) -> usize {
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
            DeviceName | Throttling | Temperatures | VramUsage | GttUsage | PowerUsage
            | FanSpeed => 0,
        }
    }

    pub fn show_peak(&self) -> bool {
        use StatType::*;
        !matches!(
            self,
            VramSize
                | GttSize
                | PowerCap
                | DeviceName
                | Throttling
                | Temperatures
                | VramUsage
                | GttUsage
                | PowerUsage
                | FanSpeed
        )
    }
}

fn power_usage_value(stats: &DeviceStats) -> Option<f64> {
    stats
        .power
        .current
        .filter(|value| *value != 0.0)
        .or(stats.power.average)
}

fn clock_level(current: Option<u64>, min: Option<u64>, max: Option<u64>) -> f64 {
    match (current, max, min) {
        (Some(cur), Some(max), Some(min)) if max > min => {
            cur.saturating_sub(min) as f64 / max.saturating_sub(min) as f64
        }
        _ => 0.0,
    }
}

fn format_current_gfxclk(value: Option<u64>) -> String {
    if let Some(value) = value {
        // If the APU/GPU does not actually support current_gfxclk, the value will be u16::MAX.
        if value >= u16::MAX as u64 || value == 0 {
            fl!(I18N, "missing-stat")
        } else {
            formatting::fmt_clockspeed(Some(value), 1.0)
        }
    } else {
        fl!(I18N, "missing-stat")
    }
}

pub fn run(args: GuiArgs) -> anyhow::Result<()> {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .parse(args.log_level.as_deref().unwrap_or_default())
        .context("Invalid log level")?;
    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    // handle panic
    let old_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        old_hook(info);

        if PANICKED.swap(true, Ordering::SeqCst) {
            return;
        }

        let panic_msg = if let Some(msg) = info.payload().downcast_ref::<&str>() {
            msg.to_string()
        } else if let Some(msg) = info.payload().downcast_ref::<String>() {
            msg.clone()
        } else {
            "Unknown panic".to_string()
        };

        let location = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown location".to_string());

        let full_msg = format!("Application panicked at {location}:\n{panic_msg}");

        let main_context = MainContext::default();
        if main_context.is_owner() {
            APP_BROKER.send(AppMsg::Crash(full_msg));
            // when panic happens in the main thread, it buble up and kills the mainLoop
            // which results in the application being unresponsive.
            // this hack "revives" it
            let loop_ = glib::MainLoop::new(Some(&main_context), false);
            glib::idle_add_local_once(move || {
                loop_.run();
            });
        } else {
            main_context.invoke_with_priority(glib::Priority::HIGH, move || {
                APP_BROKER.send(AppMsg::Crash(full_msg));
            });
        }
    }));

    // Pre-init localization
    LazyLock::force(&I18N);
    LazyLock::force(&lact_schema::i18n::LANGUAGE_LOADER);

    if let Some(existing_config) = UiConfig::load() {
        *CONFIG.write() = existing_config;
    }

    RelmApp::new(APP_ID)
        .with_broker(&APP_BROKER)
        .with_args(vec![])
        .run_async::<AppModel>(args);
    Ok(())
}
