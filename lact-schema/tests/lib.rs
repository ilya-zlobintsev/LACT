#[cfg(feature = "schema")]
use amdgpu_sysfs::gpu_handle::fan_control::FanInfo;
#[cfg(feature = "schema")]
use amdgpu_sysfs::hw_mon::Temperature;
#[cfg(feature = "schema")]
use lact_schema::{ClocksInfo, DeviceStats, FanStats, PmfwInfo};
#[cfg(feature = "schema")]
use std::collections::HashMap;

#[cfg(feature = "schema")]
use schemars::schema_for;

#[cfg(feature = "schema")]
use jsonschema::validator_for;

#[cfg(feature = "schema")]
#[test]
fn test_amd_clocks_table_schema() {
    let schema = schema_for!(ClocksInfo);
    let validator = validator_for(&serde_json::to_value(schema).unwrap())
        .expect("Failed to create JSON schema validator");

    // Test with a minimal clocks info (no table)
    let clocks_info = ClocksInfo {
        max_sclk: Some(2100),
        max_mclk: Some(1100),
        max_voltage: Some(1200),
        table: None,
    };

    // Serialize to JSON and validate
    let json_value = serde_json::to_value(&clocks_info).unwrap();
    validator
        .validate(&json_value)
        .expect("Failed to validate clocks info without table");
}

#[cfg(feature = "schema")]
#[test]
fn test_temperatures_schema() {
    let schema = schema_for!(DeviceStats);
    let validator = validator_for(&serde_json::to_value(schema).unwrap())
        .expect("Failed to create JSON schema validator");

    let temperature_cases = [
        Some(Temperature {
            current: Some(65.0),
            crit: Some(90.0),
            crit_hyst: Some(85.0),
        }),
        Some(Temperature {
            current: None,
            crit: Some(90.0),
            crit_hyst: Some(85.0),
        }),
        Some(Temperature {
            current: Some(65.0),
            crit: None,
            crit_hyst: None,
        }),
        None,
    ];

    for temp_case in &temperature_cases {
        let mut temps = HashMap::new();
        if let Some(temp) = temp_case {
            temps.insert("edge".to_string(), *temp);
        }

        let stats = DeviceStats {
            temps,
            ..Default::default()
        };

        // Serialize to JSON and validate
        let json_value = serde_json::to_value(&stats).unwrap();
        validator
            .validate(&json_value)
            .expect("Failed to validate temperatures schema");
    }
}

#[cfg(feature = "schema")]
#[test]
fn test_fan_info_schema() {
    let schema = schema_for!(FanStats);
    let validator = validator_for(&serde_json::to_value(schema).unwrap())
        .expect("Failed to create JSON schema validator");

    // Test with different pmfw info values
    let pmfw_info_cases = [
        PmfwInfo {
            acoustic_limit: Some(FanInfo {
                current: 50,
                allowed_range: Some((30, 100)),
            }),
            acoustic_target: Some(FanInfo {
                current: 60,
                allowed_range: Some((40, 100)),
            }),
            target_temp: Some(FanInfo {
                current: 70,
                allowed_range: Some((50, 100)),
            }),
            minimum_pwm: Some(FanInfo {
                current: 20,
                allowed_range: Some((10, 100)),
            }),
            zero_rpm_enable: Some(true),
            zero_rpm_temperature: Some(FanInfo {
                current: 40,
                allowed_range: Some((30, 60)),
            }),
        },
        PmfwInfo {
            acoustic_limit: None,
            acoustic_target: Some(FanInfo {
                current: 60,
                allowed_range: None,
            }),
            target_temp: None,
            minimum_pwm: Some(FanInfo {
                current: 20,
                allowed_range: Some((10, 100)),
            }),
            zero_rpm_enable: Some(false),
            zero_rpm_temperature: None,
        },
        PmfwInfo::default(),
    ];

    for pmfw_info_case in &pmfw_info_cases {
        let fan_stats = FanStats {
            pmfw_info: *pmfw_info_case,
            ..Default::default()
        };

        // Serialize to JSON and validate
        let json_value = serde_json::to_value(&fan_stats).unwrap();
        validator
            .validate(&json_value)
            .map_err(|e| e.to_string())
            .expect("Failed to validate fan info schema");
    }
}
