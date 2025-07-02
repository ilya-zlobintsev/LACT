use anyhow::{anyhow, Context};
use nix::{
    sys::stat::{umask, Mode},
    unistd::{chown, getuid, Gid, Group, User},
};
use std::{
    env, fs,
    path::{Path, PathBuf},
    str::FromStr,
};
use tokio::net::UnixListener;
use tracing::{debug, info};

use crate::{
    config,
    system::{run_command, IS_FLATBOX},
};

pub fn get_socket_path() -> PathBuf {
    let uid = getuid();

    if let Ok(path) = env::var("LACT_DAEMON_SOCKET_PATH") {
        PathBuf::from_str(&path).unwrap()
    } else if uid.is_root() {
        PathBuf::from_str("/run/lactd.sock").unwrap()
    } else {
        PathBuf::from_str(&format!("/run/user/{uid}/lactd.sock")).unwrap()
    }
}

pub fn cleanup() {
    let socket_path = get_socket_path();

    if socket_path.exists() {
        fs::remove_file(socket_path).expect("failed to remove socket");
    }
    debug!("removed socket");
}

pub fn listen() -> anyhow::Result<(UnixListener, PathBuf)> {
    let socket_path = get_socket_path();

    if socket_path.exists() {
        return Err(anyhow!(
            "Socket {socket_path:?} already exists. \
            This probably means that another instance of lact-daemon is currently running. \
            If you are sure that this is not the case, please remove the file"
        ));
    }

    let socket_mask = Mode::S_IXUSR | Mode::S_IXGRP | Mode::S_IRWXO;
    umask(socket_mask);

    let listener = UnixListener::bind(&socket_path)?;

    info!("listening on {socket_path:?}");
    Ok((listener, socket_path))
}

pub async fn set_permissions(
    socket_path: &Path,
    daemon_config: &config::Daemon,
) -> anyhow::Result<()> {
    let group = daemon_config
        .admin_group
        .as_ref()
        .map(|name| {
            Group::from_name(name)
                .context("Could not get group")?
                .with_context(|| format!("Group {name} does not exist"))
        })
        .transpose()?
        .map_or_else(Gid::current, |group| group.gid);

    let user = daemon_config
        .admin_user
        .as_ref()
        .map(|name| {
            User::from_name(name)
                .context("Could not get group")?
                .with_context(|| format!("Group {name} does not exist"))
        })
        .transpose()?
        .map(|user| user.uid);

    debug!("using gid {group} uid {user:?} for socket");

    if *IS_FLATBOX {
        let owner_arg = match user {
            Some(user) => format!("{}:{}", user.as_raw(), group.as_raw()),
            None => format!(":{}", group.as_raw()),
        };

        let path = socket_path
            .to_str()
            .expect("Invalid socket path")
            .trim_start_matches("/run/host/root");

        run_command("chown", &[&owner_arg, path])
            .await
            .context("Could not set socket permissions")?;
    } else {
        chown(socket_path, user, Some(group)).context("Could not set socket permissions")?;
    }

    Ok(())
}
