use super::confirmation_dialog::ConfirmationOptions;
use lact_schema::DeviceStats;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum AppMsg {
    Error(Rc<anyhow::Error>),
    ReloadData { full: bool },
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
