use super::ProfileWatcherEvent;
use lact_schema::{ProcessInfo, ProfileWatcherState};
use libcopes::{ProcessEventsConnector, PID};
use std::fs;
use tokio::sync::mpsc;
use tracing::{debug, error};

pub fn load_full_process_list(state: &mut ProfileWatcherState) {
    let process_entries = fs::read_dir("/proc")
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
        });

    for raw_pid in process_entries {
        let pid = PID::from(raw_pid);
        if let Ok(info) = get_pid_info(pid) {
            state.push_process(raw_pid, info);
        }
    }
}

pub fn start_listener(event_tx: mpsc::Sender<ProfileWatcherEvent>) {
    match ProcessEventsConnector::try_new() {
        Ok(connector) => {
            tokio::task::spawn_blocking(move || {
                let mut iter = connector.into_iter();
                loop {
                    if let Some(result) = iter.next() {
                        match result {
                            Ok(event) => {
                                if event_tx
                                    .blocking_send(ProfileWatcherEvent::Process(event))
                                    .is_err()
                                {
                                    debug!(
                                    "profile watcher channel closed, exiting process event listener"
                                );
                                    break;
                                }
                            }
                            Err(err) => {
                                debug!("process event error: {err}");
                            }
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
    let exe = libcopes::io::exe_reader(pid)?;
    let cmdline = libcopes::io::cmdline_reader(pid)?;
    let name = libcopes::get_process_executed_file(exe, &cmdline)
        .to_string()
        .into();

    Ok(ProcessInfo {
        name,
        cmdline: cmdline
            .to_string()
            .trim_matches(|c| c == '[' || c == ']')
            .into(),
    })
}
