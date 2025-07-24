use super::{
    confirmation_dialog::ConfirmationOptions,
    header::profile_rule_window::{profile_row::ProfileRuleRowMsg, ProfileRuleWindowMsg},
};
use lact_client::ConnectionStatusMsg;
use lact_schema::{config::ProfileHooks, request::ProfileBase, DeviceStats, ProfileRule};
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
