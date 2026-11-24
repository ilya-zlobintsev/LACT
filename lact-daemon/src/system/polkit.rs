use super::{IS_FLATBOX, run_command};
use anyhow::Context as _;
use std::{fs, path::Path};
use tracing::{error, info, warn};
use zbus_polkit::policykit1::AuthorityProxy;

const FLATPAK_POLICY_PATH: &str = "/app/share/polkit-1/actions";
const POLICY_FILE_NAME: &str = "io.github.ilya_zlobintsev.LACT.policy";

pub mod actions {
    pub const PROFILE_HOOK: &str = "io.github.ilya_zlobintsev.LACT.profile-hook";
}

#[derive(Clone)]
pub struct PolkitWrapper {
    pub proxy: AuthorityProxy<'static>,
    version: Option<i64>,
}

impl PolkitWrapper {
    pub fn supports_pidfd(&self) -> bool {
        self.version.is_some_and(|version| version >= 124)
    }

    pub fn supports_run_dir_actions(&self) -> bool {
        self.version.is_some_and(|version| version >= 126)
    }
}

pub async fn setup_polkit() -> anyhow::Result<PolkitWrapper> {
    let conn = zbus::Connection::system()
        .await
        .context("Could not establish dbus connection")?;

    let proxy = AuthorityProxy::new(&conn)
        .await
        .context("Could not connect to polkit")?;

    let raw_version = proxy.backend_version().await?;
    let version = raw_version.parse::<i64>().ok();

    let wrapper = PolkitWrapper { proxy, version };

    if *IS_FLATBOX {
        setup_flatpak_host_actions(&wrapper)
            .await
            .context("Could not install polkit policy file")?;
    }

    Ok(wrapper)
}

async fn setup_flatpak_host_actions(wrapper: &PolkitWrapper) -> anyhow::Result<()> {
    let host_target_path = if wrapper.supports_run_dir_actions() {
        "run/polkit-1/actions"
    } else {
        // It's pretty ugly to be copying a file to a persistent path,
        // but this is the only location older polkit versions read
        "usr/share/polkit-1/actions"
    };
    let target_path = Path::new("/run/host/root").join(host_target_path);

    info!(?target_path, "setting up polkit policy on the host system");

    fs::copy(
        Path::new(FLATPAK_POLICY_PATH).join(POLICY_FILE_NAME),
        target_path.join(POLICY_FILE_NAME),
    )
    .with_context(|| format!("Could not copy policy file to '{}'", target_path.display()))?;

    if let Err(err) = run_command("systemctl", &["reload", "polkit"]).await {
        warn!(%err, "could not reload polkit");
    }

    let available_actions = wrapper
        .proxy
        .enumerate_actions("")
        .await
        .context("Could not list actions")?;

    if !available_actions
        .iter()
        .any(|action| action.action_id == actions::PROFILE_HOOK)
    {
        error!("polkit action was installed successfully, but not picked up by polkit");
    }

    Ok(())
}
