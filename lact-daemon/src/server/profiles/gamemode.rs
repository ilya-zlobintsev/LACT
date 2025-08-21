use crate::system::IS_FLATBOX;
use anyhow::Context;
use lact_schema::ProfileProcessMap;
use libcopes::{PEvent, PID};
use nix::unistd::{Uid, User};
use serde::Deserialize;
use serde_json::Value;
use std::{ffi::OsString, os::unix::fs::MetadataExt, path::PathBuf, process::Stdio, rc::Rc};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    select,
    sync::{mpsc, Notify},
};
use tracing::{debug, error, info, warn};

pub const PROCESS_NAME: &str = "gamemoded";
const INTERFACE_NAME: &str = "com.feralinteractive.GameMode";

pub struct GameModeConnector {
    program_name: OsString,
    base_args: Vec<OsString>,
}

impl GameModeConnector {
    pub fn connect(process_list: &ProfileProcessMap) -> Option<Self> {
        let Some((pid, _)) = process_list
            .iter()
            .find(|(_, info)| info.name.as_ref() == PROCESS_NAME)
        else {
            info!("gamemode daemon not found");
            return None;
        };

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

        info!(
            "attempting to connect to gamemode at user {}",
            gamemode_user.name
        );

        let mut base_args: Vec<OsString> = vec![];
        let program_name = if *IS_FLATBOX {
            base_args.extend_from_slice(&["--host".into(), "busctl".into()]);
            "flatpak-spawn"
        } else {
            "busctl"
        };

        base_args.extend_from_slice(&[
            "--user".into(),
            "--machine".into(),
            format!("{}@", gamemode_user.name).into(),
            "--json".into(),
            "short".into(),
        ]);

        Some(Self {
            program_name: program_name.into(),
            base_args,
        })
    }

    pub async fn list_games(&self) -> anyhow::Result<Vec<i32>> {
        let output = Command::new(&self.program_name)
            .args(&self.base_args)
            .arg("call")
            .arg(INTERFACE_NAME)
            .arg("/com/feralinteractive/GameMode")
            .arg(INTERFACE_NAME)
            .arg("ListGames")
            .output()
            .await?;
        let response: GamesResponse =
            serde_json::from_slice(&output.stdout).with_context(|| {
                format!(
                    "Could not parse busctl output: {}{}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                )
            })?;

        Ok(response
            .data
            .into_iter()
            .flatten()
            .map(|(pid, _)| pid)
            .collect())
    }

    pub fn receieve_events(
        &self,
        stop_notify: Rc<Notify>,
    ) -> anyhow::Result<mpsc::Receiver<PEvent>> {
        let (tx, rx) = mpsc::channel(100);

        let mut child = Command::new(&self.program_name)
            .args(&self.base_args)
            .arg("monitor")
            .arg("--match")
            .arg(format!("sender={INTERFACE_NAME},type=signal"))
            .stdout(Stdio::piped())
            .spawn()?;

        let stdout = child.stdout.take().context("No child stdout")?;
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        tokio::task::spawn_local(async move {
            debug!("gamemode watcher listening");
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

            debug!("gamemode watcher task exiting");
            if let Err(err) = child.start_kill() {
                error!("could not kill gamemode watcher child: {err}");
            }
        });

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
