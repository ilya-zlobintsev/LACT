use super::ProfileWatcherEvent;
use copes::{io::connector::ProcessEventsConnector, solver::PID};
use std::fs;
use tokio::sync::mpsc;
use tracing::{debug, error};

#[allow(clippy::module_name_repetitions)]
pub struct ProcessInfo {
    pub name: String,
    pub cmdline: String,
}

pub fn load_full_process_list() -> impl Iterator<Item = (PID, ProcessInfo)> {
    fs::read_dir("/proc")
        .inspect_err(|err| error!("could not read /proc: {err}"))
        .into_iter()
        .flatten()
        .filter_map(|result| match result {
            Ok(entry) => entry
                .file_name()
                .to_str()
                .and_then(|name| name.parse::<i32>().ok()),
            Err(err) => {
                error!("could not read /proc entry: {err}");
                None
            }
        })
        .filter_map(|pid| {
            let pid = PID::from(pid);
            let info = get_pid_info(pid).ok()?;
            Some((pid, info))
        })
}

pub fn start_listener(event_tx: mpsc::Sender<ProfileWatcherEvent>) {
    match ProcessEventsConnector::try_new() {
        Ok(connector) => {
            tokio::task::spawn_blocking(move || {
                let iter = connector.into_iter();
                for result in iter {
                    match result {
                        Ok(event) => {
                            let _ = event_tx.blocking_send(ProfileWatcherEvent::Process(event));
                        }
                        Err(err) => {
                            debug!("process event error: {err}");
                        }
                    }
                }
            });
        }
        Err(err) => {
            error!("could not subscribe to process events: {err}");
        }
    }
}

pub fn get_pid_info(pid: PID) -> std::io::Result<ProcessInfo> {
    let exe = copes::io::proc::exe_reader(pid)?;
    let cmdline = copes::io::proc::cmdline_reader(pid)?;
    let executed_file = copes::solver::get_process_executed_file(exe, &cmdline).to_string();

    Ok(ProcessInfo {
        name: executed_file.to_string(),
        cmdline: cmdline.to_string(),
    })
}
