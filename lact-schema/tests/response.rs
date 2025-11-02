#[cfg(feature = "schema")]
use amdgpu_sysfs::gpu_handle::PerformanceLevel;
#[cfg(feature = "schema")]
use anyhow::anyhow;
#[cfg(feature = "schema")]
use lact_schema::Response;

#[cfg(feature = "schema")]
use schemars::schema_for;

#[cfg(feature = "schema")]
use jsonschema::validator_for;

#[cfg(feature = "schema")]
#[test]
fn test_error_schema() {
    let schema = schema_for!(Response);
    let validator = validator_for(&serde_json::to_value(schema).unwrap())
        .expect("Failed to create JSON schema validator");

    let error = anyhow!("test error")
        .context("Caused by this thing")
        .context("Caused by another thing");
    let response: Response = error.into();

    let json_value = serde_json::to_value(&response).unwrap();
    validator
        .validate(&json_value)
        .expect("Failed to validate error response");
}

#[cfg(feature = "schema")]
#[test]
fn test_power_profile_mode_schema() {
    use lact_schema::DeviceStats;

    let schema = schema_for!(DeviceStats);
    let validator = validator_for(&serde_json::to_value(schema).unwrap())
        .expect("Failed to create JSON schema validator");

    let performance_levels = [
        Some(PerformanceLevel::Auto),
        Some(PerformanceLevel::High),
        Some(PerformanceLevel::Low),
        Some(PerformanceLevel::Manual),
        None,
    ];

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
