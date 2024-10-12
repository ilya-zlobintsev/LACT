use super::process::ProcessInfo;
use copes::solver::PID;
use indexmap::IndexMap;
use nix::unistd::{geteuid, seteuid};
use std::{env, fs, os::unix::fs::MetadataExt, path::PathBuf};
use tracing::{error, info};
use zbus::{
    proxy,
    zvariant::{ObjectPath, OwnedObjectPath},
    Connection,
};

pub const PROCESS_NAME: &str = "gamemoded";
const DBUS_ADDRESS_ENV: &str = "DBUS_SESSION_BUS_ADDRESS";

#[proxy(
    interface = "com.feralinteractive.GameMode",
    default_service = "com.feralinteractive.GameMode",
    default_path = "/com/feralinteractive/GameMode"
)]
pub trait GameMode {
    #[zbus(property)]
    fn client_count(&self) -> zbus::Result<i32>;

    fn list_games(&self) -> zbus::Result<Vec<(i32, OwnedObjectPath)>>;

    #[zbus(signal)]
    fn game_registered(&self, pid: i32, object_path: ObjectPath<'_>) -> zbus::Result<()>;

    #[zbus(signal)]
    fn game_unregistered(&self, pid: i32, object_path: ObjectPath<'_>) -> zbus::Result<()>;
}

#[proxy(
    interface = "com.feralinteractive.GameMode.Game",
    default_service = "com.feralinteractive.GameMode"
)]
pub trait GameModeGame {
    #[zbus(property)]
    fn process_id(&self) -> zbus::Result<i32>;

    #[zbus(property)]
    fn executable(&self) -> zbus::Result<String>;
}

pub async fn connect(
    process_list: &IndexMap<PID, ProcessInfo>,
) -> Option<(Connection, GameModeProxy<'static>)> {
    let mut address = None;
    let mut gamemode_uid = None;

    if let Ok(raw_address) = env::var(DBUS_ADDRESS_ENV) {
        address = Some(raw_address);
    } else if let Some((pid, _)) = process_list
        .iter()
        .find(|(_, info)| info.name == PROCESS_NAME)
    {
        let process_path = PathBuf::from(*pid);
        let metadata = process_path
            .metadata()
            .map_err(|err| error!("could not read gamemode process metadata: {err}"))
            .ok()?;

        gamemode_uid = Some(metadata.uid());

        let gamemode_env = fs::read(process_path.join("environ"))
            .map_err(|err| error!("could not read gamemode process env: {err}"))
            .ok()?;

        let dbus_addr_env = gamemode_env
            .split(|c| *c == b'\0')
            .filter_map(|pair| std::str::from_utf8(pair).ok())
            .filter_map(|pair| pair.split_once('='))
            .find(|(key, _)| *key == DBUS_ADDRESS_ENV);
        match dbus_addr_env {
            Some((_, env_address)) => {
                address = Some(env_address.to_owned());
            }
            None => {
                error!("could not find DBus address env variable on gamemode's process");
            }
        }
    }

    if let Some(address) = address {
        info!("attempting to connect to gamemode using session {address}");

        let builder = zbus::conn::Builder::address(address.as_str())
            .map_err(|err| error!("could not construct DBus connection: {err}"))
            .ok()?;

        let connection_result = match gamemode_uid {
            Some(gamemode_uid) => {
                info!("gamemode session uid: {gamemode_uid}");
                // It is very important that the euid gets reset back to the original,
                // regardless of what's happening with the dbus connection
                let original_uid = geteuid();

                seteuid(gamemode_uid.into())
                    .map_err(|err| error!("failed to set euid to gamemode's uid: {err}"))
                    .ok()?;

                let connection_result = builder.build().await;

                // If this fails then something is terribly wrong and we cannot continue
                seteuid(original_uid).expect("Failed to reset euid back to original");
                connection_result
            }
            None => builder.build().await,
        };

        let connection = connection_result
            .map_err(|err| error!("could not connect to DBus: {err}"))
            .ok()?;

        let proxy = GameModeProxy::new(&connection)
            .await
            .map_err(|err| info!("could not connect to gamemode: {err}"))
            .ok()?;
        let client_count = proxy
            .client_count()
            .await
            .map_err(|err| error!("could not fetch gamemode client count: {err}"))
            .ok()?;

        info!("connected to gamemode daemon, games active: {client_count}");

        Some((connection, proxy))
    } else {
        info!("gamemode daemon not found");
        None
    }
}
