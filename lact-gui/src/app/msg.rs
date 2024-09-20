use super::confirmation_dialog::ConfirmationOptions;
use lact_schema::DeviceStats;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum AppMsg {
    Error(Rc<anyhow::Error>),
    ReloadData,
    Stats(DeviceStats),
    ApplyChanges,
    RevertChanges,
    ShowGraphsWindow,
    DumpVBios,
    DebugSnapshot,
    DisableOverdrive,
    ResetConfig,
    AskConfirmation(ConfirmationOptions, Box<AppMsg>),
}

impl AppMsg {
    pub fn ask_confirmation(
        inner: AppMsg,
        title: &'static str,
        message: &'static str,
        buttons_type: gtk::ButtonsType,
    ) -> Self {
        Self::AskConfirmation(
            ConfirmationOptions {
                title,
                message,
                buttons_type,
            },
            Box::new(inner),
        )
    }
}
