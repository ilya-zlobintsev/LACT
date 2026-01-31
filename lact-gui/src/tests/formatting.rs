use crate::app::formatting::{self, ByteUnit, Mono};
use amdgpu_sysfs::hw_mon::Temperature;
use lact_schema::{DeviceStats, TemperatureEntry};
use std::collections::{BTreeMap, HashMap};

#[test]
fn mono_display_formats_values() {
    assert_eq!(
        Mono::int(-5).to_string(),
        "<span font_family='monospace'>-5</span>"
    );
    assert_eq!(
        Mono::uint(42_u64).to_string(),
        "<span font_family='monospace'>42</span>"
    );
    assert_eq!(
        Mono::float(1.2, 3).to_string(),
        "<span font_family='monospace'>1.200</span>"
    );
}

#[test]
fn fmt_fan_speed_formats_rpm_and_percent() {
    let mut stats = DeviceStats::default();
    stats.fan.pwm_current = Some(128);
    stats.fan.speed_current = Some(3000);

    assert_eq!(
        formatting::fmt_fan_speed(&stats),
        Some("<b><span font_family='monospace'>3000</span> RPM (<span font_family='monospace'>50</span>%)</b>".to_string())
    );
}

#[test]
fn fmt_fan_speed_formats_percent_only() {
    let mut stats = DeviceStats::default();
    stats.fan.pwm_current = Some(255);

    assert_eq!(
        formatting::fmt_fan_speed(&stats),
        Some("<b><span font_family='monospace'>100</span>%</b>".to_string())
    );
}

#[test]
fn fmt_throttling_text_formats_details() {
    let mut stats = DeviceStats::default();
    let mut throttle_info = BTreeMap::new();
    throttle_info.insert(
        "Thermal".to_string(),
        vec!["GPU".to_string(), "VRAM".to_string()],
    );
    throttle_info.insert("Power".to_string(), vec![]);
    stats.throttle_info = Some(throttle_info);

    assert_eq!(
        formatting::fmt_throttling_text(&stats),
        "Power, Thermal (GPU, VRAM)"
    );
}

#[test]
fn fmt_temperature_text_sorts_labels() {
    let mut stats = DeviceStats::default();
    stats.temps = HashMap::from([
        (
            "junction".to_string(),
            TemperatureEntry {
                value: Temperature {
                    crit: None,
                    crit_hyst: None,
                    current: Some(80.0),
                },
                display_only: false,
            },
        ),
        (
            "edge".to_string(),
            TemperatureEntry {
                value: Temperature {
                    crit: None,
                    crit_hyst: None,
                    current: Some(55.0),
                },
                display_only: false,
            },
        ),
    ]);

    assert_eq!(
        formatting::fmt_temperature_text(&stats),
        Some(
            "edge: <span font_family='monospace'>55</span>°C, junction: <span font_family='monospace'>80</span>°C"
                .to_string()
        )
    );
}

#[test]
fn fmt_clockspeed_uses_localized_unit() {
    assert_eq!(
        formatting::fmt_clockspeed(Some(1000), 1.0),
        "<span font_family='monospace'>1000</span> MHz"
    );

    assert_eq!(
        formatting::fmt_clockspeed(Some(1000), 3.0),
        "<span font_family='monospace'>3000</span> MHz"
    );
}

#[test]
fn fmt_timestamp_to_dt_formats_time() {
    let timestamp_ms = 0;
    assert_eq!(formatting::fmt_timestamp_to_dt(&timestamp_ms), "00:00:00");
}

#[test]
fn fmt_human_bytes_formats_auto_and_fixed_units() {
    assert_eq!(formatting::fmt_human_bytes(1024, None), "1024 bytes");
    assert_eq!(formatting::fmt_human_bytes(2049, None), "2.0 KiB");
    assert_eq!(
        formatting::fmt_human_bytes(1_073_741_824, Some(ByteUnit::Gibibyte)),
        "1 GiB"
    );
    assert_eq!(
        formatting::fmt_human_bytes(1_610_612_736, Some(ByteUnit::Gibibyte)),
        "1.5 GiB"
    );
}
