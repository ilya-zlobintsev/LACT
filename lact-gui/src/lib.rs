mod app;
mod config;

use anyhow::Context;
use app::{AppModel, APP_BROKER};
use config::UiConfig;
use lact_schema::args::GuiArgs;
use relm4::{RelmApp, SharedState};
use tracing::metadata::LevelFilter;
use tracing_subscriber::EnvFilter;

static CONFIG: SharedState<UiConfig> = SharedState::new();

const GUI_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const APP_ID: &str = "io.github.lact-linux";

pub fn run(args: GuiArgs) -> anyhow::Result<()> {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .parse(args.log_level.as_deref().unwrap_or_default())
        .context("Invalid log level")?;
    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    if let Some(existing_config) = UiConfig::load() {
        *CONFIG.write() = existing_config;
    }

    RelmApp::new(APP_ID)
        .with_broker(&APP_BROKER)
        .with_args(vec![])
        .run_async::<AppModel>(args);
    Ok(())
}
