mod app;
mod config;

use std::sync::LazyLock;

use anyhow::Context;
use app::{AppModel, APP_BROKER};
use config::UiConfig;
use i18n_embed::fluent::{fluent_language_loader, FluentLanguageLoader};
use lact_schema::{args::GuiArgs, i18n};
use relm4::{RelmApp, SharedState};
use rust_embed::RustEmbed;
use tracing::metadata::LevelFilter;
use tracing_subscriber::EnvFilter;

static CONFIG: SharedState<UiConfig> = SharedState::new();

const GUI_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const APP_ID: &str = "io.github.ilya_zlobintsev.LACT";
pub const REPO_URL: &str = "https://github.com/ilya-zlobintsev/LACT";

pub(crate) static I18N: LazyLock<FluentLanguageLoader> =
    LazyLock::new(|| i18n::loader(fluent_language_loader!(), &Localizations));

#[derive(RustEmbed)]
#[folder = "i18n"]
pub struct Localizations;

pub fn run(args: GuiArgs) -> anyhow::Result<()> {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .parse(args.log_level.as_deref().unwrap_or_default())
        .context("Invalid log level")?;
    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    // Pre-init localization
    LazyLock::force(&I18N);
    LazyLock::force(&lact_schema::i18n::LANGUAGE_LOADER);

    if let Some(existing_config) = UiConfig::load() {
        *CONFIG.write() = existing_config;
    }

    RelmApp::new(APP_ID)
        .with_broker(&APP_BROKER)
        .with_args(vec![])
        .run_async::<AppModel>(args);
    Ok(())
}
