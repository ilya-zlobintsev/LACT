pub mod crash_page;
pub mod displays_page;
pub mod info_page;
pub mod oc_page;
pub mod software_page;
pub mod thermals_page;

use lact_schema::{DeviceInfo, DeviceStats};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageId {
    Info,
    Oc,
    Thermals,
    Software,
    Displays,
    Crash,
}

impl PageId {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info_page",
            Self::Oc => "oc_page",
            Self::Thermals => "thermals_page",
            Self::Software => "software_page",
            Self::Displays => "displays_page",
            Self::Crash => "crash_page",
        }
    }
}

#[derive(Debug, Clone)]
pub enum PageUpdate {
    Info(Arc<DeviceInfo>),
    Stats(Arc<DeviceStats>),
}
