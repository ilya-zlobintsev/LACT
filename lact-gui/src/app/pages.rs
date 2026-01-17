pub mod crash_page;
pub mod info_page;
pub mod oc_adjustment;
pub mod oc_page;
pub mod software_page;
pub mod thermals_page;

use lact_schema::{DeviceInfo, DeviceStats};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum PageUpdate {
    Info(Arc<DeviceInfo>),
    Stats(Arc<DeviceStats>),
}
