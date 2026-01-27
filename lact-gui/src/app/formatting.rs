use std::fmt;

use i18n_embed_fl::fl;
use lact_schema::DeviceStats;

use crate::I18N;

/// Displays numeric values with monospace font.
/// Should be used in oftent updated data.
pub enum Mono {
    #[allow(dead_code)]
    Int(i64),
    UInt(u64),
    Float {
        value: f64,
        precision: usize,
    },
}

impl Mono {
    #[allow(dead_code)]
    pub fn int(value: impl Into<i64>) -> Self {
        Self::Int(value.into())
    }

    pub fn uint(value: impl Into<u64>) -> Self {
        Self::UInt(value.into())
    }

    pub fn float(value: impl Into<f64>, precision: usize) -> Self {
        Self::Float {
            value: value.into(),
            precision,
        }
    }
}

impl fmt::Display for Mono {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<span font_family='monospace'>")?;
        match *self {
            Self::Int(v) => write!(f, "{v}")?,
            Self::UInt(v) => write!(f, "{v}")?,
            Self::Float { value, precision } => write!(f, "{value:.prec$}", prec = precision)?,
        }
        f.write_str("</span>")
    }
}

pub fn fmt_fan_speed(stats: &DeviceStats) -> Option<String> {
    let fan_percent = stats
        .fan
        .pwm_current
        .map(|current_pwm| ((current_pwm as f64 / u8::MAX as f64) * 100.0).round() as u64);

    match (stats.fan.speed_current.map(u64::from), fan_percent) {
        (Some(rpm), Some(percent)) => Some(format!(
            "<b>{} RPM ({}%)</b>",
            Mono::uint(rpm),
            Mono::uint(percent)
        )),
        (Some(rpm), None) => Some(format!("<b>{} RPM</b>", Mono::uint(rpm))),
        (None, Some(percent)) => Some(format!("<b>{}%</b>", Mono::uint(percent))),
        (None, None) => None,
    }
}

pub fn fmt_throttling_text(stats: &DeviceStats) -> String {
    match &stats.throttle_info {
        Some(throttle_info) => {
            if throttle_info.is_empty() {
                fl!(I18N, "no-throttling")
            } else {
                let type_text: Vec<String> = throttle_info
                    .iter()
                    .map(|(throttle_type, details)| {
                        if details.is_empty() {
                            throttle_type.to_string()
                        } else {
                            format!("{throttle_type} ({})", details.join(", "))
                        }
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

pub fn fmt_temperature_text(stats: &DeviceStats) -> Option<String> {
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

pub fn fmt_clockspeed(clock_mhz: Option<u64>, ratio: f64) -> String {
    format!(
        "{} {}",
        Mono::float(clock_mhz.unwrap_or(0) as f64 * ratio, 0),
        fl!(I18N, "mhz")
    )
}

pub fn fmt_timestamp_to_dt(timestamp_ms: &i64) -> String {
    let date_time = chrono::DateTime::from_timestamp_millis(*timestamp_ms).unwrap();
    date_time.format("%H:%M:%S").to_string()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum ByteUnit {
    Bytes = 0,
    Kibibyte = 1,
    Mebibyte = 2,
    Gibibyte = 3,
}

impl ByteUnit {
    const ALL: [Self; 4] = [Self::Bytes, Self::Kibibyte, Self::Mebibyte, Self::Gibibyte];

    fn label(self) -> String {
        match self {
            ByteUnit::Bytes => fl!(I18N, "bytes"),
            ByteUnit::Kibibyte => fl!(I18N, "kibibyte"),
            ByteUnit::Mebibyte => fl!(I18N, "mebibyte"),
            ByteUnit::Gibibyte => fl!(I18N, "gibibyte"),
        }
    }

    fn scale(self, bytes: u64) -> f64 {
        bytes as f64 / 1024.0_f64.powi(self as i32)
    }
}

pub fn fmt_human_bytes(bytes: u64, unit: Option<ByteUnit>) -> String {
    let (size, unit) = if let Some(unit) = unit {
        (unit.scale(bytes), unit)
    } else {
        let mut size = bytes as f64;
        let mut i = 0;
        while size > 2048.0 && i < ByteUnit::ALL.len() - 1 {
            size /= 1024.0;
            i += 1;
        }

        (size, ByteUnit::ALL[i])
    };

    let label = unit.label();
    format!("{size:.1$} {}", label, (size.fract() != 0.0) as usize)
}
