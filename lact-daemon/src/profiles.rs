mod gamemode;
mod process;

use crate::{config::Config, server::handler::Handler};
use copes::solver::{PEvent, PID};
use futures::StreamExt;
use indexmap::{IndexMap, IndexSet};
use lact_schema::{ProcessProfileRule, ProfileRule};
use process::ProcessInfo;
use std::{rc::Rc, time::Instant};
use tokio::{select, sync::mpsc};
use tracing::{error, info, trace, warn};

struct WatcherState {
    process_list: IndexMap<PID, ProcessInfo>,
    gamemode_games: IndexSet<PID>,
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
        gamemode_games: IndexSet::new(),
    };
    info!("loaded {} processes", state.process_list.len());

    let (event_tx, mut event_rx) = mpsc::channel(128);

    process::start_listener(event_tx.clone());

    if let Some((_conn, proxy)) = gamemode::connect(&state.process_list).await {
        match proxy.list_games().await {
            Ok(games) => {
                for (pid, _) in games {
                    state.gamemode_games.insert(pid.into());
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

    update_profile(&state, &handler).await;

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
                state.process_list.shift_remove(&pid);
            }
            ProfileWatcherEvent::Gamemode(PEvent::Exec(pid)) => {
                state.gamemode_games.insert(pid);
            }
            ProfileWatcherEvent::Gamemode(PEvent::Exit(pid)) => {
                state.gamemode_games.shift_remove(&pid);
            }
        }

        update_profile(&state, &handler).await;
    }
}

async fn update_profile(state: &WatcherState, handler: &Handler) {
    let started_at = Instant::now();
    let new_profile = evaluate_current_profile(state, &handler.config.borrow());
    trace!("evaluated profile rules in {:?}", started_at.elapsed());

    if handler.config.borrow().current_profile != new_profile {
        match &new_profile {
            Some(name) => info!("setting current profile to {name}"),
            None => info!("setting default profile"),
        }

        if let Err(err) = handler.set_profile(new_profile, false).await {
            error!("failed to apply profile: {err:#}");
        }
    }
}

/// Returns the new active profile
fn evaluate_current_profile(state: &WatcherState, config: &Config) -> Option<Rc<str>> {
    // TODO: fast path to re-evaluate only a single event and not the whole state?
    for pid in state.gamemode_games.iter().rev() {
        for (profile_name, profile) in &config.profiles {
            if let Some(ProfileRule::Gamemode(process_filter)) = &profile.rule {
                match process_filter {
                    Some(filter) => {
                        if let Some(process) = state.process_list.get(pid) {
                            if process_rule_matches(filter, process) {
                                return Some(profile_name.clone());
                            }
                        }
                    }
                    None => return Some(profile_name.clone()),
                }
            }
        }
    }

    for process in state.process_list.values().rev() {
        for (profile_name, profile) in &config.profiles {
            if let Some(ProfileRule::Process(rule)) = &profile.rule {
                if process_rule_matches(rule, process) {
                    return Some(profile_name.clone());
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
    use indexmap::{IndexMap, IndexSet};
    use lact_schema::{ProcessProfileRule, ProfileRule};
    use pretty_assertions::assert_eq;
    use std::{collections::HashMap, rc::Rc};

    #[test]
    fn evaluate_basic_profile() {
        let mut state = WatcherState {
            process_list: IndexMap::from([(
                PID::from(1),
                ProcessInfo {
                    name: "game1".to_owned(),
                    cmdline: String::new(),
                },
            )]),
            gamemode_games: IndexSet::new(),
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
