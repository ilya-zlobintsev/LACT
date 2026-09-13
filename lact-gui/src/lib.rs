mod app;
mod config;
mod service_setup;

use std::{
    panic,
    sync::{LazyLock, atomic::AtomicBool, atomic::Ordering},
};

use anyhow::Context;
use app::{APP_BROKER, AppModel, msg::AppMsg};
use config::UiConfig;
use i18n_embed::{
    LanguageLoader,
    fluent::{FluentLanguageLoader, fluent_language_loader},
    unic_langid::LanguageIdentifier,
};
use lact_schema::{args::GuiArgs, i18n};
use relm4::{
    RelmApp, SharedState,
    gtk::{glib, glib::MainContext},
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
    i18n::loader(
        fluent_language_loader!(),
        &Localizations,
        cfg!(test).then(|| vec!["en-US".parse().unwrap()]),
    )
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

    if let Some(existing_config) = UiConfig::load() {
        *CONFIG.write() = existing_config;
    }

    // Initialize system localization before applying the saved override.
    LazyLock::force(&I18N);
    LazyLock::force(&i18n::LANGUAGE_LOADER);
    if let Some(language) = &CONFIG.read().language
        && let Err(err) = select_language(language, &I18N, &i18n::LANGUAGE_LOADER)
    {
        tracing::warn!(%language, "Could not apply saved language: {err:#}");
    }

    RelmApp::new(APP_ID)
        .with_broker(&APP_BROKER)
        .with_args(vec![])
        .run_async::<AppModel>(args);
    Ok(())
}

fn select_language(
    language: &str,
    gui_loader: &FluentLanguageLoader,
    schema_loader: &FluentLanguageLoader,
) -> anyhow::Result<()> {
    let language: LanguageIdentifier = language.parse().context("Invalid language identifier")?;
    anyhow::ensure!(
        gui_loader
            .available_languages(&Localizations)?
            .contains(&language),
        "Language is not available in GUI localizations"
    );
    let requested = [language];
    i18n_embed::select(gui_loader, &Localizations, &requested)?;
    i18n_embed::select(schema_loader, &i18n::Localizations, &requested)?;
    Ok(())
}
