use lact_schema::ProcessMap;
use libcopes::PID;
use nix::unistd::{geteuid, seteuid, Uid};
use std::{
    env, fs,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};
use tracing::{error, info};
use zbus::{
    proxy,
    zvariant::{ObjectPath, OwnedObjectPath},
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

pub async fn connect(process_list: &ProcessMap) -> Option<GameModeProxy<'static>> {
    let address;
    let gamemode_uid;

    if let Ok(raw_address) = env::var(DBUS_ADDRESS_ENV) {
        match Path::new(&raw_address.trim_start_matches("unix:path=")).metadata() {
            Ok(metadata) => {
                gamemode_uid = metadata.uid();
            }
            Err(err) => {
                error!("could not read DBus socket metadata from {raw_address}: {err}");
                gamemode_uid = geteuid().into();
            }
        }

        address = raw_address;
    } else if let Some((pid, _)) = process_list
        .iter()
        .find(|(_, info)| info.name.as_ref() == PROCESS_NAME)
    {
        let pid = PID::from(*pid);
        let process_path = PathBuf::from(pid);
        let metadata = process_path
            .metadata()
            .map_err(|err| error!("could not read gamemode process metadata: {err}"))
            .ok()?;

        gamemode_uid = metadata.uid();

        let gamemode_env = fs::read(process_path.join("environ"))
            .map_err(|err| error!("could not read gamemode process env: {err}"))
            .ok()?;

        let dbus_addr_env = gamemode_env
            .split(|c| *c == b'\0')
            .filter_map(|pair| std::str::from_utf8(pair).ok())
            .filter_map(|pair| pair.split_once('='))
            .find(|(key, _)| *key == DBUS_ADDRESS_ENV);

        if let Some((_, env_address)) = dbus_addr_env {
            address = env_address.to_owned();
        } else {
            error!("could not find DBus address env variable on gamemode's process");
            return None;
        }
    } else {
        info!("gamemode daemon not found");
        return None;
    }

    info!("attempting to connect to gamemode on DBus address {address}");

    let builder = zbus::conn::Builder::address(address.as_str())
        .map_err(|err| error!("could not construct DBus connection: {err}"))
        .ok()?;

    let connection_result;

    // It is very important that the euid gets reset back to the original,
    // regardless of what's happening with the dbus connection
    let original_uid = geteuid();
    let gamemode_uid = Uid::from(gamemode_uid);

    if original_uid == gamemode_uid {
        connection_result = builder.build().await;
    } else {
        info!("gamemode session uid: {gamemode_uid}");

        seteuid(gamemode_uid)
            .map_err(|err| error!("failed to set euid to gamemode's uid: {err}"))
            .ok()?;

        connection_result = builder.build().await;

        // If this fails then something is terribly wrong and we cannot continue
        seteuid(original_uid).expect("Failed to reset euid back to original");
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

    Some(proxy)
}
