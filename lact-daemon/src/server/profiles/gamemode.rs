use crate::system::IS_FLATBOX;
use anyhow::Context;
use lact_schema::ProfileProcessMap;
use libcopes::{PEvent, PID};
use nix::unistd::{Uid, User};
use serde::Deserialize;
use serde_json::Value;
use std::{
    collections::BTreeMap, ffi::OsString, fs, os::unix::fs::MetadataExt, path::PathBuf,
    process::Stdio, rc::Rc,
};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    select,
    sync::{Notify, mpsc},
};
use tracing::{debug, error, info, warn};

pub const PROCESS_NAME: &str = "gamemoded";
const INTERFACE_NAME: &str = "com.feralinteractive.GameMode";
const DBUS_ADDRESS_ENV_PREFIX: &str = "DBUS_SESSION_BUS_ADDRESS=";

pub struct GameModeConnector {
    program_name: OsString,
    args_sets: BTreeMap<u32, Vec<OsString>>,
}

impl GameModeConnector {
    pub fn new(process_list: &ProfileProcessMap) -> Option<Self> {
        let mut base_args: Vec<OsString> = vec![];
        let program_name = if *IS_FLATBOX {
            base_args.extend_from_slice(&["--host".into(), "sudo".into()]);
            "flatpak-spawn"
        } else {
            "sudo"
        };

        let mut args_sets = BTreeMap::new();

        for (pid, info) in process_list {
            if info.name.as_ref() == PROCESS_NAME {
                let mut base_args = base_args.clone();

                let pid = PID::from(*pid);
                let process_path = PathBuf::from(pid);
                let metadata = process_path
                    .metadata()
                    .map_err(|err| error!("could not read gamemode process metadata: {err}"))
                    .ok()?;

                let gamemode_uid = Uid::from_raw(metadata.uid());
                let gamemode_user = User::from_uid(gamemode_uid)
                    .inspect_err(|err| error!("could not fetch gamemode process user: {err}"))
                    .ok()
                    .flatten()?;

                let dbus_addr_env = fs::read(process_path.join("environ"))
                    .map_err(|err| error!("could not read gamemode process env: {err}"))
                    .ok()
                    .and_then(|gamemode_env| {
                        gamemode_env
                            .split(|c| *c == b'\0')
                            .filter_map(|pair| std::str::from_utf8(pair).ok())
                            .find(|line| line.starts_with(DBUS_ADDRESS_ENV_PREFIX))
                            .context("Could not find DBUS env on gamemode process")
                            .map(str::to_owned)
                            .ok()
                    })
                    .unwrap_or_else(|| {
                        // Fallback value if actual address can't be read from env
                        format!("{DBUS_ADDRESS_ENV_PREFIX}unix:path=/run/user/{gamemode_uid}/bus")
                    });

                info!("connecting to gamemode with '{dbus_addr_env}'");

                base_args.extend_from_slice(&[
                    "-u".into(),
                    gamemode_user.name.into(),
                    dbus_addr_env.into(),
                    "busctl".into(),
                    "--user".into(),
                    "--json".into(),
                    "short".into(),
                ]);
                args_sets.insert(gamemode_uid.as_raw(), base_args);
            }
        }

        if args_sets.is_empty() {
            info!("gamemode daemon not found");
            None
        } else {
            Some(Self {
                program_name: program_name.into(),
                args_sets,
            })
        }
    }

    pub async fn list_games(&self) -> anyhow::Result<Vec<i32>> {
        let mut pids = Vec::new();

        for base_args in self.args_sets.values() {
            let mut cmd = Command::new(&self.program_name);
            cmd.args(base_args)
                .arg("call")
                .arg(INTERFACE_NAME)
                .arg("/com/feralinteractive/GameMode")
                .arg(INTERFACE_NAME)
                .arg("ListGames")
                .kill_on_drop(true);
            debug!("running {cmd:?}");
            let output = cmd.output().await?;
            let response: GamesResponse =
                serde_json::from_slice(&output.stdout).with_context(|| {
                    format!(
                        "Could not parse busctl output: {}{}",
                        String::from_utf8_lossy(&output.stdout),
                        String::from_utf8_lossy(&output.stderr)
                    )
                })?;

            let instance_pids = response.data.into_iter().flatten().map(|(pid, _)| pid);
            pids.extend(instance_pids);
        }

        Ok(pids)
    }

    pub fn receieve_events(
        &self,
        stop_notify: &Rc<Notify>,
    ) -> anyhow::Result<mpsc::Receiver<PEvent>> {
        let (tx, rx) = mpsc::channel(100);

        for (uid, base_args) in &self.args_sets {
            let mut child = Command::new(&self.program_name)
                .args(base_args)
                .arg("monitor")
                .arg("--match")
                .arg(format!("sender={INTERFACE_NAME},type=signal"))
                .stdout(Stdio::piped())
                .kill_on_drop(true)
                .spawn()?;

            let stdout = child.stdout.take().context("No child stdout")?;
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();

            let stop_notify = stop_notify.clone();
            let tx = tx.clone();
            let uid = *uid;

            tokio::task::spawn_local(async move {
                debug!("gamemode watcher listening for user {uid}");
                loop {
                    select! {
                        result = lines.next_line() => {
                            match result {
                                Ok(Some(line)) => match serde_json::from_str::<SignalMessage>(&line) {
                                    Ok(msg) => match msg.member.as_str() {
                                        "GameRegistered" => {
                                            if let Some(pid) = msg.extract_pid() {
                                                if tx.send(PEvent::Exec(pid.into())).await.is_err() {
                                                    break;
                                                }
                                            } else {
                                                warn!("could not parse gamemode payload: {line}");
                                            }
                                        }
                                        "GameUnregistered" => {
                                            if let Some(pid) = msg.extract_pid() {
                                                if tx.send(PEvent::Exit(pid.into())).await.is_err() {
                                                    break;
                                                }
                                            } else {
                                                warn!("could not parse gamemode payload: {line}");
                                            }
                                        }
                                        _ => (),
                                    },
                                    Err(err) => warn!("could not parse gamemode signal: {err}: {line}"),
                                },
                                Ok(None) => (),
                                Err(err) => {
                                    error!("gamemode watcher error: {err}");
                                    break;
                                }
                            }
                        },
                        () = stop_notify.notified() => {
                            break;
                        }
                    }
                }

                debug!("gamemode watcher task for user {uid} exiting");
                if let Err(err) = child.start_kill() {
                    error!("could not kill gamemode watcher child for user {uid}: {err}");
                }
            });
        }

        Ok(rx)
    }
}

#[derive(Deserialize)]
struct GamesResponse {
    data: Vec<Vec<(i32, String)>>,
}

#[derive(Deserialize)]
struct SignalMessage {
    pub member: String,
    pub payload: SignalMessagePayload,
}

impl SignalMessage {
    fn extract_pid(&self) -> Option<i32> {
        self.payload
            .data
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(Value::as_i64)
            .and_then(|val| i32::try_from(val).ok())
    }
}

#[derive(Deserialize)]
struct SignalMessagePayload {
    data: Value,
}
