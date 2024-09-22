pub mod app;

use anyhow::Context;
use app::AppModel;
use lact_schema::args::GuiArgs;
use relm4::RelmApp;
use tracing::metadata::LevelFilter;
use tracing_subscriber::EnvFilter;

const GUI_VERSION: &str = env!("CARGO_PKG_VERSION");
const APP_ID: &str = "io.github.lact-linux";

pub fn run(args: GuiArgs) -> anyhow::Result<()> {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .parse(args.log_level.as_deref().unwrap_or_default())
        .context("Invalid log level")?;
    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    let app = RelmApp::new(APP_ID).with_args(vec![]);
    app.run_async::<AppModel>(args);
    Ok(())
}
