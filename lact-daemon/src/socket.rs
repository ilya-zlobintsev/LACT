use anyhow::{anyhow, Context};
use nix::{
    sys::stat::{umask, Mode},
    unistd::{chown, getuid, Gid, Group, User},
};
use std::{fs, path::PathBuf, str::FromStr};
use tokio::net::UnixListener;
use tracing::{debug, info};

pub fn get_socket_path() -> PathBuf {
    let uid = getuid();
    if uid.is_root() {
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

pub fn listen(admin_group: Option<&str>, admin_user: Option<&str>) -> anyhow::Result<UnixListener> {
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

    let group = admin_group
        .map(|name| {
            Group::from_name(name)
                .context("Could not get group")?
                .with_context(|| format!("Group {name} does not exist"))
        })
        .transpose()?
        .map_or_else(Gid::current, |group| group.gid);

    let user = admin_user
        .map(|name| {
            User::from_name(name)
                .context("Could not get group")?
                .with_context(|| format!("Group {name} does not exist"))
        })
        .transpose()?
        .map(|user| user.uid);

    debug!("using gid {group} uid {user:?} for socket");

    chown(&socket_path, user, Some(group))?;

    info!("listening on {socket_path:?}");
    Ok(listener)
}
