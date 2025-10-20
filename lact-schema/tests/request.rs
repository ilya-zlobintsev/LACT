#[cfg(feature = "schema")]
use amdgpu_sysfs::gpu_handle::{PerformanceLevel, PowerLevelKind};
#[cfg(feature = "schema")]
use jsonschema::validator_for;
#[cfg(feature = "schema")]
use lact_schema::DeviceStats;

#[cfg(feature = "schema")]
use schemars::schema_for;

#[cfg(feature = "schema")]
#[test]
fn test_performance_level_schema() {
    let performance_levels = [
        Some(PerformanceLevel::Auto),
        Some(PerformanceLevel::High),
        Some(PerformanceLevel::Low),
        Some(PerformanceLevel::Manual),
        None,
    ];

    let schema = schema_for!(DeviceStats);
    let validator = validator_for(&serde_json::to_value(schema).unwrap())
        .expect("Failed to create JSON schema validator");

    for level in &performance_levels {
        let stats = DeviceStats {
            performance_level: level.clone(),
            ..Default::default()
        };

        let json_value = serde_json::to_value(&stats).unwrap();
        validator
            .validate(&json_value)
            .expect("Failed to validate performance level");
    }
}

#[cfg(feature = "schema")]
#[test]
fn test_power_level_kind_schema() {
    use lact_schema::Request;

    let power_level_kinds = [
        PowerLevelKind::CoreClock,
        PowerLevelKind::MemoryClock,
        PowerLevelKind::SOCClock,
        PowerLevelKind::FabricClock,
        PowerLevelKind::DCEFClock,
        PowerLevelKind::PcieSpeed,
    ];

    let schema = schema_for!(Request);
    let validator = validator_for(&serde_json::to_value(schema).unwrap())
        .expect("Failed to create JSON schema validator");

    for kind in &power_level_kinds {
        let request = Request::SetEnabledPowerStates {
            id: "test-gpu",
            kind: *kind,
            states: vec![0, 1],
        };

        let json_value = serde_json::to_value(&request).unwrap();
        validator
            .validate(&json_value)
            .expect("Failed to validate power level kind");
    }
}
