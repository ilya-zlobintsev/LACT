use anyhow::anyhow;
use nix::{
    sys::stat::{umask, Mode},
    unistd::{chown, getuid, Gid, Group},
};
use std::{fs, path::PathBuf, str::FromStr};
use tokio::net::UnixListener;
use tracing::{debug, info};

pub fn get_socket_path() -> PathBuf {
    let uid = getuid();
    if uid.is_root() {
        PathBuf::from_str("/var/run/lactd.sock").unwrap()
    } else {
        PathBuf::from_str(&format!("/var/run/user/{uid}/lactd.sock")).unwrap()
    }
}

pub fn cleanup() {
    let socket_path = get_socket_path();

    if socket_path.exists() {
        fs::remove_file(socket_path).expect("failed to remove socket");
    }
    debug!("removed socket");
}

pub fn listen(admin_groups: &[String]) -> anyhow::Result<UnixListener> {
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

    chown(&socket_path, None, Some(socket_gid(admin_groups)))?;

    info!("listening on {socket_path:?}");
    Ok(listener)
}

fn socket_gid(admin_groups: &[String]) -> Gid {
    if getuid().is_root() {
        // Check if the group exists
        for group_name in admin_groups {
            if let Ok(Some(group)) = Group::from_name(group_name) {
                return group.gid;
            }
        }

        if let Ok(Some(group)) = Group::from_gid(Gid::from_raw(1000)) {
            group.gid
        } else {
            Gid::current()
        }
    } else {
        Gid::current()
    }
}
