use anyhow::{anyhow, Context};
use tracing::{debug, error, info, warn};
use zbus::{proxy, Connection};

const CONFLICTING_ACTIONS: [&str; 1] = ["amdgpu_dpm"];
const MIN_PPD_MINOR_VERSION: u32 = 30;

pub async fn setup() {
    let conn = match Connection::system().await {
        Ok(conn) => conn,
        Err(err) => {
            warn!("could not establish DBus connection: {err}");
            return;
        }
    };

    match PowerProfilesDaemonProxy::new(&conn).await {
        Ok(ppd_client) => {
            if let Err(err) = disable_conflicting_actions(&ppd_client).await {
                warn!("power-profiles-daemon detected, but conflicting actions could not be disabled: {err:#}");
            }
        }
        Err(err) => {
            debug!("could not connect to power-profiles-daemon: {err}");
        }
    }
}

async fn disable_conflicting_actions(client: &PowerProfilesDaemonProxy<'_>) -> anyhow::Result<()> {
    let version = client.version().await?;
    debug!("connected to power-profiles-daemon {version}");

    let (_major, minor) = version
        .split_once('.')
        .with_context(|| format!("Could not parse version string '{version}'"))?;
    let minor = minor
        .parse::<u32>()
        .context("Could not parse minor version")?;

    if minor < MIN_PPD_MINOR_VERSION {
        return Err(anyhow!(
            "daemon version {version} is older than minimum required for actions configuration"
        ));
    }

    let current_actions = client
        .actions_info()
        .await
        .context("Could not list actions")?;

    for action_map in current_actions {
        if let Some(name) = action_map
            .get("Name")
            .and_then(|value| value.downcast_ref::<String>().ok())
        {
            if CONFLICTING_ACTIONS.contains(&name.as_str()) {
                match action_map
                    .get("Enabled")
                    .and_then(|enabled| enabled.downcast_ref::<bool>().ok())
                {
                    Some(enabled) => {
                        if enabled {
                            match client.set_action_enabled(&name, false).await {
                                Ok(()) => {
                                    info!(
                                        "disabled conflicting power-profiles-daemon action {name}"
                                    );
                                }
                                Err(err) => {
                                    error!("could not disable conflicting power-profiles-daemon action {name}: {err}");
                                }
                            }
                        } else {
                            info!("conflicting power-profiles-daemon action {name} is already disabled");
                        }
                    }
                    None => {
                        error!("could not check status for power-profiles-daemon action {name}: {action_map:?}");
                    }
                }
            }
        }
    }

    Ok(())
}

#[proxy(
    interface = "org.freedesktop.UPower.PowerProfiles",
    default_service = "org.freedesktop.UPower.PowerProfiles",
    default_path = "/org/freedesktop/UPower/PowerProfiles"
)]
trait PowerProfilesDaemon {
    /// SetActionEnabled method
    fn set_action_enabled(&self, action: &str, enabled: bool) -> zbus::Result<()>;

    /// Actions property
    #[zbus(property)]
    fn actions(&self) -> zbus::Result<Vec<String>>;

    /// ActionsInfo property
    #[zbus(property)]
    fn actions_info(
        &self,
    ) -> zbus::Result<Vec<std::collections::HashMap<String, zbus::zvariant::OwnedValue>>>;

    /// ActiveProfile property
    #[zbus(property)]
    fn active_profile(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn set_active_profile(&self, value: &str) -> zbus::Result<()>;

    /// Version property
    #[zbus(property)]
    fn version(&self) -> zbus::Result<String>;
}
