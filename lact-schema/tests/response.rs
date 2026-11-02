#[cfg(feature = "schema")]
use amdgpu_sysfs::gpu_handle::PerformanceLevel;
#[cfg(feature = "schema")]
use anyhow::anyhow;
#[cfg(feature = "schema")]
use lact_schema::{Response, ResponseData};

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
            performance_level: *level,
            ..Default::default()
        };

        let json_value = serde_json::to_value(&stats).unwrap();
        validator
            .validate(&json_value)
            .expect("Failed to validate performance level");
    }
}

/// Every payload the daemon can send must validate against the response
/// schema, otherwise clients generated from it would reject valid responses.
#[cfg(feature = "schema")]
#[test]
fn test_all_response_payloads_validate() {
    use std::collections::BTreeMap;

    use amdgpu_sysfs::gpu_handle::PowerLevelId;
    use amdgpu_sysfs::gpu_handle::fan_control::FanInfo;
    use amdgpu_sysfs::gpu_handle::power_profile_mode::PowerProfileModesTable;
    use amdgpu_sysfs::hw_mon::Temperature;
    use indexmap::IndexMap;
    use lact_schema::{
        ActivePowerStates, ClocksInfo, DeviceApiInfo, DeviceInfo, DeviceListEntry, DeviceStats,
        DeviceType, DisplaysInfo, LinkInfo, NvidiaThermalInfo, Pong, PowerStates, ProcessList,
        ProcessProfileRule, ProfileRule, ProfilesInfo, SystemInfo, TemperatureEntry, VersionInfo,
        config::{ClocksConfiguration, GpuConfig, Profile},
    };
    use nvml_wrapper::enums::device::PowerMizerMode;

    let schema = schema_for!(Response);
    let schema_value = serde_json::to_value(&schema).unwrap();
    let validator = validator_for(&schema_value).expect("Failed to create JSON schema validator");

    // One representative payload per `ResponseData` variant.
    let mut variants: Vec<(&str, ResponseData)> = Vec::new();
    // Additional shapes for variants that serialize differently.
    let mut extra_payloads: Vec<(&str, serde_json::Value)> = Vec::new();

    macro_rules! variant {
        ($name:literal, $value:expr) => {
            variants.push(($name, ResponseData::from($value)));
        };
    }

    macro_rules! extra {
        ($name:literal, $value:expr) => {
            extra_payloads.push((
                $name,
                serde_json::to_value(ResponseData::from($value)).unwrap(),
            ));
        };
    }

    variant!("no_data", ());
    variant!("integer", 42_u64);
    variant!("evaluate_profile_rule", true);
    variant!("generate_snapshot", "lact-snapshot.tar.gz".to_owned());
    variant!("vbios_dump", vec![0_u8, 1, 2, 255]);
    variant!(
        "list_devices",
        vec![DeviceListEntry {
            id: "1002:73bf-1da2:e437-0000:0c:00.0".to_owned(),
            name: Some("Radeon RX 7900 XTX".to_owned()),
            device_type: DeviceType::Dedicated,
        }]
    );
    variant!(
        "system_info",
        SystemInfo {
            version: VersionInfo::current(),
            distro: Some("CachyOS".to_owned()),
            kernel_version: "6.18.1".to_owned(),
            amdgpu_overdrive_enabled: Some(true),
            amdgpu_params_configurator: None,
        }
    );
    variant!(
        "device_info",
        DeviceInfo {
            pci_info: None,
            api_info: DeviceApiInfo::default(),
            driver: "amdgpu".to_owned(),
            vbios_version: Some("113-D7020100-102".to_owned()),
            link_info: LinkInfo::default(),
            drm_info: None,
            flags: vec![],
        }
    );
    variant!(
        "device_stats",
        DeviceStats {
            temps: IndexMap::from([(
                "edge".to_owned(),
                TemperatureEntry {
                    value: Temperature {
                        current: Some(65.0),
                        crit: Some(100.0),
                        crit_hyst: Some(95.0),
                    },
                    primary: true,
                    display_only: false,
                },
            )]),
            performance_level: Some(PerformanceLevel::High),
            active_power_mizer_mode: Some(PowerMizerMode::Auto),
            supported_power_mizer_modes: Some(
                vec![PowerMizerMode::Adaptive, PowerMizerMode::Auto,]
            ),
            active_power_states: Some(ActivePowerStates {
                core: Some(PowerLevelId::Index(2)),
                memory: Some(PowerLevelId::Sleep),
                pcie: None,
            }),
            nvidia_thermal_info: NvidiaThermalInfo {
                target_temp: Some(FanInfo {
                    current: 70,
                    allowed_range: Some((50, 90)),
                }),
                target_temp_default: Some(80),
            },
            ..Default::default()
        }
    );
    variant!("process_list", ProcessList::default());
    variant!("list_profiles", ProfilesInfo::default());
    variant!("get_power_states", PowerStates::default());
    variant!("displays_info", DisplaysInfo::default());
    variant!(
        "device_power_profile_modes",
        PowerProfileModesTable {
            modes: BTreeMap::new(),
            value_names: vec!["CUSTOM".to_owned()],
            active: 0,
        }
    );
    variant!("device_clocks_info", ClocksInfo::default());
    variant!("device_api_info", DeviceApiInfo::default());
    variant!("get_profile", Some(Profile::default()));
    variant!(
        "get_gpu_config",
        Some(GpuConfig {
            fan_control_enabled: true,
            power_cap: Some(250.0),
            performance_level: Some(PerformanceLevel::High),
            power_mizer_mode: Some(PowerMizerMode::PreferMaximumPerformance),
            uma_carveout: Some(1),
            clocks_configuration: ClocksConfiguration {
                max_core_clock: Some(2900),
                min_core_clock: Some(500),
                gpu_clock_offsets: IndexMap::from([(0_u32, 100), (1, -50)]),
                ..Default::default()
            },
            ..Default::default()
        })
    );
    variant!("ping", Pong);

    extra!("device_stats_default", DeviceStats::default());
    extra!("get_profile_none", None::<Profile>);
    extra!("get_gpu_config_none", None::<GpuConfig>);
    extra!(
        "list_profiles_populated",
        ProfilesInfo {
            profiles: IndexMap::from([
                ("default".to_owned(), None),
                (
                    "gaming".to_owned(),
                    Some(ProfileRule::Process(ProcessProfileRule {
                        name: "game".into(),
                        args: None,
                    })),
                ),
            ]),
            ..Default::default()
        }
    );

    let extra_count = extra_payloads.len();
    let payloads = variants
        .iter()
        .map(|(name, data)| (*name, serde_json::to_value(data).unwrap()))
        .chain(extra_payloads);

    let mut validated = 0;
    for (name, payload) in payloads {
        validated += 1;

        let envelope = serde_json::json!({ "status": "ok", "data": payload });

        if let Err(error) = validator.validate(&envelope) {
            panic!(
                "Response payload `{name}` does not match the generated schema: {error}\n{envelope}"
            );
        }
    }

    // Every `ResponseData` variant has to be present in the generated union,
    // and each one is covered by a representative payload above.
    let union_len = schema_value["$defs"]["ResponseData"]["anyOf"]
        .as_array()
        .expect("ResponseData schema is not a union")
        .len();

    assert_eq!(
        union_len,
        variants.len(),
        "the response payload list is out of sync with ResponseData"
    );
    assert_eq!(validated, variants.len() + extra_count);
}
