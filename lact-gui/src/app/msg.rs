use super::confirmation_dialog::ConfirmationOptions;
use lact_client::ConnectionStatusMsg;
use lact_schema::{request::ProfileBase, DeviceStats, ProfileRule};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum AppMsg {
    Error(Arc<anyhow::Error>),
    ReloadData {
        full: bool,
    },
    Stats(Arc<DeviceStats>),
    ApplyChanges,
    RevertChanges,
    SettingsChanged,
    ResetClocks,
    ResetPmfw,
    ShowGraphsWindow,
    DumpVBios,
    DebugSnapshot,
    EnableOverdrive,
    DisableOverdrive,
    ResetConfig,
    ReloadProfiles {
        include_state: bool,
    },
    SelectProfile {
        profile: Option<String>,
        auto_switch: bool,
    },
    CreateProfile(String, ProfileBase),
    DeleteProfile(String),
    MoveProfile(String, usize),
    EvaluateProfile(ProfileRule),
    SetProfileRule {
        name: String,
        rule: Option<ProfileRule>,
    },
    ImportProfile,
    ExportProfile(Option<String>),
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
