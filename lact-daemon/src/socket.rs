use nix::{
    sys::stat::{umask, Mode},
    unistd::{chown, getuid, Gid, Group},
};
use std::{fs, path::PathBuf, str::FromStr};
use tokio::net::UnixListener;
use tracing::{debug, info};

const ADMIN_GROUPS: &[&str] = &["wheel", "sudo"];

pub fn get_socket_path() -> PathBuf {
    let uid = getuid();
    if uid.is_root() {
        PathBuf::from_str("/var/run/lactd.sock").unwrap()
    } else {
        PathBuf::from_str(&format!("/var/run/user/{}/lactd.sock", uid)).unwrap()
    }
}

pub fn cleanup() {
    let socket_path = get_socket_path();

    if socket_path.exists() {
        fs::remove_file(socket_path).expect("failed to remove socket")
    }
    debug!("removed socket");
}

pub async fn listen() -> anyhow::Result<UnixListener> {
    let socket_path = get_socket_path();

    if socket_path.exists() {
        fs::remove_file(&socket_path)?;
    }

    let socket_mask = Mode::S_IXUSR | Mode::S_IXGRP | Mode::S_IRWXO;
    umask(socket_mask);

    let listener = UnixListener::bind(&socket_path)?;

    chown(&socket_path, None, Some(socket_gid()))?;

    info!("listening on {socket_path:?}");
    Ok(listener)
}

fn socket_gid() -> Gid {
    if getuid().is_root() {
        // Check if the group exists
        for group_name in ADMIN_GROUPS {
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
