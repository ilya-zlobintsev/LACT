use amdgpu_sysfs::gpu_handle::power_profile_mode::PowerProfileModesTable;
use derive_more::{From, TryInto};
#[cfg(feature = "schema")]
use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::{Deserialize, Serialize};

use crate::{
    ClocksInfo, DeviceApiInfo, DeviceInfo, DeviceListEntry, DeviceStats, DisplaysInfo, Pong,
    PowerStates, ProcessList, ProfilesInfo, SystemInfo,
    config::{GpuConfig, Profile},
};

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(tag = "status", content = "data", rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)]
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

/// Union of every possible response payload.
///
/// The payload is adjacently tagged: `type` identifies the variant and `data`
/// holds its value, mirroring how [`Request`](crate::Request) uses
/// `command`/`args`. Adjacent tagging is required because several payloads are
/// not JSON objects (`null`, numbers, strings, arrays) and therefore cannot be
/// tagged in-place.
#[derive(Serialize, Deserialize, Debug, From, TryInto)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[try_into(owned, ref, ref_mut)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)]
pub enum ResponseData {
    NoData(()),
    Integer(u64),
    EvaluateProfileRule(bool),
    GenerateSnapshot(String),
    VbiosDump(Vec<u8>),
    ListDevices(Vec<DeviceListEntry>),
    SystemInfo(SystemInfo),
    DeviceInfo(DeviceInfo),
    DeviceStats(DeviceStats),
    ProcessList(ProcessList),
    ListProfiles(ProfilesInfo),
    GetPowerStates(PowerStates),
    DisplaysInfo(DisplaysInfo),
    #[cfg_attr(
        feature = "schema",
        schemars(schema_with = "power_profile_mode_schema")
    )]
    DevicePowerProfileModes(PowerProfileModesTable),
    DeviceClocksInfo(ClocksInfo),
    DeviceApiInfo(DeviceApiInfo),
    GetProfile(Option<Profile>),
    GetGpuConfig(Option<GpuConfig>),
    Ping(Pong),
}

#[cfg(feature = "schema")]
fn power_profile_mode_schema(_rec: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "object",
    })
}

#[cfg(feature = "schema")]
fn error_schema(_rec: &mut SchemaGenerator) -> Schema {
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
