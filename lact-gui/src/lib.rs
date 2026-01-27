mod app;
mod config;
#[cfg(test)]
mod tests;

use std::{
    panic,
    sync::{atomic::AtomicBool, atomic::Ordering, LazyLock},
};

use anyhow::Context;
use app::{msg::AppMsg, AppModel, APP_BROKER};
use config::UiConfig;
use i18n_embed::fluent::{fluent_language_loader, FluentLanguageLoader};
use lact_schema::{args::GuiArgs, i18n};
use relm4::{
    gtk::{glib, glib::MainContext},
    RelmApp, SharedState,
};
use rust_embed::RustEmbed;
use tracing::metadata::LevelFilter;
use tracing_subscriber::EnvFilter;

static CONFIG: SharedState<UiConfig> = SharedState::new();
static PANICKED: AtomicBool = AtomicBool::new(false);

const GUI_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const APP_ID: &str = "io.github.ilya_zlobintsev.LACT";
pub const REPO_URL: &str = "https://github.com/ilya-zlobintsev/LACT";

pub(crate) static I18N: LazyLock<FluentLanguageLoader> = LazyLock::new(|| {
    #[cfg(test)]
    {
        std::env::set_var("LANGUAGE", "en");
        std::env::set_var("LC_ALL", "en_US.UTF-8");
        std::env::set_var("LANG", "en_US.UTF-8");
    }

    i18n::loader(fluent_language_loader!(), &Localizations)
});

#[derive(RustEmbed)]
#[folder = "i18n"]
pub struct Localizations;

pub fn run(args: GuiArgs) -> anyhow::Result<()> {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .parse(args.log_level.as_deref().unwrap_or_default())
        .context("Invalid log level")?;
    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    // handle panic
    let old_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        old_hook(info);

        if PANICKED.swap(true, Ordering::SeqCst) {
            return;
        }

        let panic_msg = if let Some(msg) = info.payload().downcast_ref::<&str>() {
            msg.to_string()
        } else if let Some(msg) = info.payload().downcast_ref::<String>() {
            msg.clone()
        } else {
            "Unknown panic".to_string()
        };

        let location = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown location".to_string());

        let full_msg = format!("Application panicked at {location}:\n{panic_msg}");

        let main_context = MainContext::default();
        if main_context.is_owner() {
            APP_BROKER.send(AppMsg::Crash(full_msg));
            // when panic happens in the main thread, it buble up and kills the mainLoop
            // which results in the application being unresponsive.
            // this hack "revives" it
            let loop_ = glib::MainLoop::new(Some(&main_context), false);
            glib::idle_add_local_once(move || {
                loop_.run();
            });
        } else {
            main_context.invoke_with_priority(glib::Priority::HIGH, move || {
                APP_BROKER.send(AppMsg::Crash(full_msg));
            });
        }
    }));

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
