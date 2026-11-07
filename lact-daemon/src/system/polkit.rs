use crate::system::IS_FLATBOX;
use anyhow::Context as _;
use std::{fs, path::Path};
use tracing::info;
use zbus_polkit::policykit1::AuthorityProxy;

const FLATPAK_POLICY_PATH: &str = "/app/share/polkit-1/actions";
const POLICY_FILE_NAME: &str = "io.github.ilya_zlobintsev.LACT.policy";

pub async fn setup_polkit() -> anyhow::Result<AuthorityProxy<'static>> {
    let conn = zbus::Connection::system()
        .await
        .context("Could not establish dbus connection")?;

    let proxy = AuthorityProxy::new(&conn)
        .await
        .context("Could not connect to polkit")?;

    if *IS_FLATBOX {
        setup_flatpak_host_actions(&proxy)
            .await
            .context("Could not install polkit policy file")?;
    }

    Ok(proxy)
}

async fn setup_flatpak_host_actions(proxy: &AuthorityProxy<'static>) -> anyhow::Result<()> {
    let raw_version = proxy.backend_version().await?;
    let version: i64 = raw_version
        .parse()
        .with_context(|| format!("Could not parse polkit version '{raw_version}'"))?;

    let host_target_path = if version >= 126 {
        "run/polkit-1/actions"
    } else {
        // It's pretty ugly to be copying a file to a persistent path,
        // but this is the only location older polkit versions read
        "usr/share/polkit-1/actions"
    };
    let target_path = Path::new("/run/host/root").join(host_target_path);

    fs::create_dir_all(&target_path)
        .with_context(|| format!("Could not create policy dir '{}'", target_path.display()))?;

    info!(
        target_path = target_path.display().to_string(),
        "setting up polkit policy on the host system"
    );

    fs::copy(
        Path::new(FLATPAK_POLICY_PATH).join(POLICY_FILE_NAME),
        target_path.join(POLICY_FILE_NAME),
    )
    .with_context(|| format!("Could not copy policy file to '{}'", target_path.display()))?;

    // TODO: validate here

    Ok(())
}
