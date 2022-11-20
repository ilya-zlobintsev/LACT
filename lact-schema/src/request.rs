use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(tag = "command", content = "args", rename_all = "snake_case")]
pub enum Request<'a> {
    Ping,
    ListDevices,
    DeviceInfo { id: &'a str },
    DeviceStats { id: &'a str },
    SetFanControl { id: &'a str, enabled: bool },
    SetPowerLimit { id: &'a str, limit: f64 },
}
