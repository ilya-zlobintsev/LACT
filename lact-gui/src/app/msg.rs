use super::{
    confirmation_dialog::ConfirmationOptions,
    header::profile_rule_window::{ProfileRuleWindowMsg, profile_row::ProfileRuleRowMsg},
};
use lact_client::ConnectionStatusMsg;
use lact_schema::{DeviceStats, ProfileRule, config::ProfileHooks, request::ProfileBase};
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
    ShowProcessMonitor,
    DumpVBios,
    DebugSnapshot,
    ShowOverdriveDialog,
    EnableOverdrive,
    DisableOverdrive,
    ResetConfig,
    FetchProcessList,
    ReloadProfiles {
        state_sender: Option<relm4::Sender<ProfileRuleRowMsg>>,
    },
    SelectProfile {
        profile: Option<String>,
        auto_switch: bool,
    },
    CreateProfile(String, ProfileBase),
    DeleteProfile(String),
    MoveProfile(String, usize),
    RenameProfile(String, String),
    EvaluateProfile(ProfileRule, relm4::Sender<ProfileRuleWindowMsg>),
    SetProfileRule {
        name: String,
        rule: Option<ProfileRule>,
        hooks: ProfileHooks,
    },
    ImportProfile,
    ExportProfile(Option<String>),
    ConnectionStatus(ConnectionStatusMsg),
    AskConfirmation(ConfirmationOptions, Box<AppMsg>),
    Crash(String),
}

impl AppMsg {
    pub fn ask_confirmation(
        inner: AppMsg,
        title: String,
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
