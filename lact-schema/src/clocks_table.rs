use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct NvidiaClocksTable {
    pub gpc_offset: Option<i32>,
    pub gpc_offset_range: Option<(i32, i32)>,
    pub gpc_max: Option<i32>,
    pub mem_offset: Option<i32>,
    pub mem_offset_range: Option<(i32, i32)>,
    pub mem_max: Option<i32>,
}
