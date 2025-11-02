use amdgpu_sysfs::gpu_handle::power_profile_mode::PowerProfileModesTable;
use derive_more::{From, TryInto};
#[cfg(feature = "schema")]
use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::{
    config::{GpuConfig, Profile},
    ClocksInfo, DeviceInfo, DeviceListEntry, DeviceStats, Pong, PowerStates, ProcessList,
    ProfilesInfo, SystemInfo,
};

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(tag = "status", content = "data", rename_all = "snake_case")]
pub enum Response {
    Ok(ResponseData),
    #[cfg_attr(feature = "schema", schemars(schema_with = "error_schema"))]
    Error(serde_error::Error),
}

impl From<anyhow::Error> for Response {
    fn from(value: anyhow::Error) -> Self {
        Response::Error(serde_error::Error::new(&*value))
    }
}

impl From<serde_error::Error> for Response {
    fn from(value: serde_error::Error) -> Self {
        Response::Error(value)
    }
}

#[derive(Serialize, Deserialize, Debug, From, TryInto)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[try_into(owned, ref, ref_mut)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum ResponseData {
    Ping(Pong),
    ListDevices(Vec<DeviceListEntry>),
    SystemInfo(SystemInfo),
    DeviceInfo(DeviceInfo),
    DeviceStats(DeviceStats),
    DeviceClocksInfo(ClocksInfo),
    #[cfg_attr(
        feature = "schema",
        schemars(schema_with = "power_profile_mode_schema")
    )]
    DevicePowerProfileModes(PowerProfileModesTable),
    Integer(u64),
    GetPowerStates(PowerStates),
    VbiosDump(Vec<u8>),
    ListProfiles(ProfilesInfo),
    GetProfile(Option<Profile>),
    NoData(()),
    EvaluateProfileRule(bool),
    GetGpuConfig(Option<GpuConfig>),
    ProcessList(ProcessList),
    GenerateSnapshot(String),
}

#[cfg(feature = "schema")]
fn power_profile_mode_schema(_gen: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "object",
    })
}

#[cfg(feature = "schema")]
fn error_schema(_gen: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "object",
        "properties": {
            "description": {
                "type": "string"
            },
            "source": {
                "type": "object",
                "properties": {
                    "description": {
                        "type": "string"
                    },
                    "source": {
                        "type": "object"
                    }
                }
            }
        },
    })
}
