mod gamemode;
mod process;

use crate::{config::Config, server::handler::Handler};
use copes::solver::{PEvent, PID};
use futures::StreamExt;
use lact_schema::{ProcessProfileRule, ProfileRule};
use process::ProcessInfo;
use std::{collections::HashMap, rc::Rc};
use tokio::{select, sync::mpsc};
use tracing::{error, info, trace, warn};

struct WatcherState {
    process_list: HashMap<PID, ProcessInfo>,
    gamemode_games: HashMap<PID, ProcessInfo>,
}

#[derive(Debug)]
enum ProfileWatcherEvent {
    Process(PEvent),
    Gamemode(PEvent),
}

pub async fn run_watcher(handler: Handler) {
    let process_list = process::load_full_process_list().collect();

    let mut state = WatcherState {
        process_list,
        gamemode_games: HashMap::new(),
    };
    info!("loaded {} processes", state.process_list.len());

    let (event_tx, mut event_rx) = mpsc::channel(128);

    process::start_listener(event_tx.clone());

    if let Some((_conn, proxy)) = gamemode::connect(&state.process_list).await {
        match proxy.list_games().await {
            Ok(games) => {
                for (pid, _) in games {
                    match process::get_pid_info(pid.into()) {
                        Ok(data) => {
                            state.gamemode_games.insert(pid.into(), data);
                        }
                        Err(err) => error!("could not get info for gamemode game {pid}: {err}"),
                    }
                }
            }
            Err(err) => {
                error!("could not list gamemode games: {err}");
            }
        }

        match (
            proxy.receive_game_registered().await,
            proxy.receive_game_unregistered().await,
        ) {
            (Ok(mut registered_stream), Ok(mut unregistered_stream)) => {
                let event_tx = event_tx.clone();

                tokio::task::spawn_local(async move {
                    loop {
                        let mut event = None;

                        select! {
                            Some(registered_event) = registered_stream.next() => {
                                match registered_event.args() {
                                    Ok(args) => {
                                        event = Some(PEvent::Exec(args.pid.into()));
                                    }
                                    Err(err) => error!("could not get event args: {err}"),
                                }
                            },
                            Some(unregistered_event) = unregistered_stream.next() => {
                                match unregistered_event.args() {
                                    Ok(args) => {
                                        event = Some(PEvent::Exit(args.pid.into()));
                                    }
                                    Err(err) => error!("could not get event args: {err}"),
                                }
                            },
                        };

                        if let Some(event) = event {
                            let _ = event_tx.send(ProfileWatcherEvent::Gamemode(event)).await;
                        }
                    }
                });
            }
            err_info => {
                error!("Could not get gamemode event stream: {err_info:?}");
            }
        }
    }

    while let Some(event) = event_rx.recv().await {
        trace!("profile watcher event: {event:?}");
        match event {
            ProfileWatcherEvent::Process(PEvent::Exec(pid)) => match process::get_pid_info(pid) {
                Ok(data) => {
                    state.process_list.insert(pid, data);
                }
                Err(err) => {
                    warn!("could not get info for process {pid}: {err}");
                }
            },
            ProfileWatcherEvent::Process(PEvent::Exit(pid)) => {
                state.process_list.remove(&pid);
            }
            ProfileWatcherEvent::Gamemode(PEvent::Exec(pid)) => match process::get_pid_info(pid) {
                Ok(data) => {
                    state.gamemode_games.insert(pid, data);
                }
                Err(err) => {
                    warn!("could not get info for process {pid}: {err}");
                }
            },
            ProfileWatcherEvent::Gamemode(PEvent::Exit(pid)) => {
                state.gamemode_games.remove(&pid);
            }
        }

        evaluate_current_profile(&state, &handler.config.borrow());
    }
}

/// Returns the new active profile
fn evaluate_current_profile(state: &WatcherState, config: &Config) -> Option<Rc<str>> {
    // TODO: fast path to re-evaluate only a single event and not the whole state?
    for (profile_name, profile) in &config.profiles {
        if let Some(rule) = &profile.rule {
            match rule {
                ProfileRule::Process(rule) => {
                    for process in state.process_list.values() {
                        if process_rule_matches(rule, process) {
                            return Some(profile_name.clone());
                        }
                    }
                }
                ProfileRule::Gamemode(rule) => {
                    if !state.gamemode_games.is_empty() {
                        match rule {
                            Some(process_rule) => {
                                for process in state.process_list.values() {
                                    if process_rule_matches(process_rule, process) {
                                        return Some(profile_name.clone());
                                    }
                                }
                            }
                            None => return Some(profile_name.clone()),
                        }
                    }
                }
            }
        }
    }

    None
}

fn process_rule_matches(rule: &ProcessProfileRule, process: &ProcessInfo) -> bool {
    process.name == rule.name
        && rule
            .args
            .as_ref()
            .map_or(true, |wanted_args| process.cmdline.contains(wanted_args))
}

#[cfg(test)]
mod tests {
    use super::{process::ProcessInfo, WatcherState};
    use crate::{
        config::{Config, Profile},
        profiles::evaluate_current_profile,
    };
    use copes::solver::PID;
    use lact_schema::{ProcessProfileRule, ProfileRule};
    use pretty_assertions::assert_eq;
    use std::{collections::HashMap, rc::Rc};

    #[test]
    fn evaluate_basic_profile() {
        let mut state = WatcherState {
            process_list: HashMap::from([(
                PID::from(1),
                ProcessInfo {
                    name: "game1".to_owned(),
                    cmdline: String::new(),
                },
            )]),
            gamemode_games: HashMap::new(),
        };

        let mut config = Config::default();
        config.profiles.insert(
            "1".into(),
            Profile {
                gpus: HashMap::new(),
                rule: Some(ProfileRule::Process(ProcessProfileRule {
                    name: "game1".to_owned(),
                    args: None,
                })),
            },
        );
        config.profiles.insert(
            "2".into(),
            Profile {
                gpus: HashMap::new(),
                rule: Some(ProfileRule::Process(ProcessProfileRule {
                    name: "game2".to_owned(),
                    args: None,
                })),
            },
        );

        assert_eq!(
            Some(Rc::from("1")),
            evaluate_current_profile(&state, &config)
        );

        "game2".clone_into(&mut state.process_list.get_mut(&PID::from(1)).unwrap().name);
        assert_eq!(
            Some(Rc::from("2")),
            evaluate_current_profile(&state, &config)
        );

        state
            .process_list
            .get_mut(&PID::from(1))
            .unwrap()
            .name
            .clear();
        assert_eq!(None, evaluate_current_profile(&state, &config));
    }
}
