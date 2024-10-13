use super::confirmation_dialog::ConfirmationOptions;
use lact_client::ConnectionStatusMsg;
use lact_schema::{request::ProfileBase, DeviceStats};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum AppMsg {
    Error(Arc<anyhow::Error>),
    ReloadData {
        full: bool,
    },
    Stats(DeviceStats),
    ApplyChanges,
    RevertChanges,
    ResetClocks,
    ResetPmfw,
    ShowGraphsWindow,
    DumpVBios,
    DebugSnapshot,
    EnableOverdrive,
    DisableOverdrive,
    ResetConfig,
    ReloadProfiles,
    SelectProfile {
        profile: Option<String>,
        auto_switch: bool,
    },
    CreateProfile(String, ProfileBase),
    DeleteProfile(String),
    MoveProfile(String, usize),
    ConnectionStatus(ConnectionStatusMsg),
    AskConfirmation(ConfirmationOptions, Box<AppMsg>),
}

impl AppMsg {
    pub fn ask_confirmation(
        inner: AppMsg,
        title: &'static str,
        message: impl Into<String>,
        buttons_type: gtk::ButtonsType,
    ) -> Self {
        Self::AskConfirmation(
            ConfirmationOptions {
                title,
                message: message.into(),
                buttons_type,
            },
            Box::new(inner),
        )
    }
}
