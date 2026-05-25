//! # D-Bus interface proxy for: `org.freedesktop.systemd1.Manager`
use anyhow::Context;
use zbus::{proxy, zvariant::OwnedObjectPath};

const UNIT_NAME: &str = "lactd.service";

pub async fn connect_unit_proxy() -> anyhow::Result<UnitProxy<'static>> {
    let conn = zbus::Connection::system()
        .await
        .context("Could not establish DBus connection")?;

    let manager = ManagerProxy::new(&conn)
        .await
        .context("Could not connect to systemd manager interface")?;

    let path = manager
        .get_unit(UNIT_NAME)
        .await
        .context("Could not get lact systemd unit")?;

    let unit = UnitProxy::builder(&conn)
        .path(path)?
        .build()
        .await
        .context("Could not connect to systemd unit interface")?;

    Ok(unit)
}

#[proxy(
    interface = "org.freedesktop.systemd1.Manager",
    default_service = "org.freedesktop.systemd1",
    default_path = "/org/freedesktop/systemd1"
)]
pub trait Manager {
    #[zbus(allow_interactive_auth)]
    fn get_unit(&self, name: &str) -> zbus::Result<OwnedObjectPath>;

    #[zbus(allow_interactive_auth)]
    fn enable_unit_files(
        &self,
        files: &[&str],
        runtime: bool,
        force: bool,
    ) -> zbus::Result<(bool, Vec<(String, String, String)>)>;
}

#[proxy(
    interface = "org.freedesktop.systemd1.Unit",
    default_service = "org.freedesktop.systemd1"
)]
pub trait Unit {
    /// Restart method
    #[zbus(allow_interactive_auth)]
    fn restart(&self, mode: &str) -> zbus::Result<OwnedObjectPath>;

    /// Start method
    #[zbus(allow_interactive_auth)]
    fn start(&self, mode: &str) -> zbus::Result<OwnedObjectPath>;

    /// Stop method
    #[zbus(allow_interactive_auth)]
    fn stop(&self, mode: &str) -> zbus::Result<OwnedObjectPath>;
}
