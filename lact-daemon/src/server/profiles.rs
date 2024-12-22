mod gamemode;
mod process;

use crate::server::handler::Handler;
use copes::solver::{PEvent, PID};
use futures::StreamExt;
use indexmap::{IndexMap, IndexSet};
use lact_schema::{ProcessProfileRule, ProfileRule};
use process::ProcessInfo;
use std::{
    rc::Rc,
    time::{Duration, Instant},
};
use tokio::{
    select,
    sync::{mpsc, Notify},
    time::sleep,
};
use tracing::{error, info, trace, warn};

const PROFILE_WATCHER_MIN_DELAY_MS: u64 = 50;
const PROFILE_WATCHER_MAX_DELAY_MS: u64 = 500;

struct WatcherState {
    process_list: IndexMap<PID, ProcessInfo>,
    gamemode_games: IndexSet<PID>,
}

#[derive(Debug)]
enum ProfileWatcherEvent {
    Process(PEvent),
    Gamemode(PEvent),
}

pub async fn run_watcher(handler: Handler, stop_notify: Rc<Notify>) {
    let profile_rules = handler
        .config
        .borrow()
        .profiles
        .iter()
        .filter_map(|(name, profile)| {
            let rule = profile.rule.as_ref()?;
            Some((name.clone(), rule.clone()))
        })
        .collect::<Vec<_>>();

    let process_list = process::load_full_process_list().collect();

    let mut state = WatcherState {
        process_list,
        gamemode_games: IndexSet::new(),
    };
    info!("loaded {} processes", state.process_list.len());

    let (event_tx, mut event_rx) = mpsc::channel(128);

    process::start_listener(event_tx.clone());

    let mut gamemode_task = None;
    if let Some(gamemode_proxy) = gamemode::connect(&state.process_list).await {
        match gamemode_proxy.list_games().await {
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
            gamemode_proxy.receive_game_registered().await,
            gamemode_proxy.receive_game_unregistered().await,
        ) {
            (Ok(mut registered_stream), Ok(mut unregistered_stream)) => {
                let event_tx = event_tx.clone();

                let handle = tokio::task::spawn_local(async move {
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
                gamemode_task = Some(handle);
            }
            err_info => {
                error!("Could not get gamemode event stream: {err_info:?}");
            }
        }
    }

    update_profile(&state, &handler, &profile_rules).await;

    loop {
        select! {
            () = stop_notify.notified() => break,
            Some(event) = event_rx.recv() => {
                handle_profile_event(&event, &mut state, &handler);

                // It is very common during system usage that multiple processes start at the same time, or there are processes
                // that start and exit right away.
                // Due to this, it does not make sense to re-evaluate profile rules as soon as there is a process event.
                // Instead, we accumulate multiple events that come in quick succession, and only evaluate the rules once.
                //
                // After getting an event we wait for a period of time (the minimum delay option).
                // If there are no new events since, rules are evaluated. If there are,
                // the timer is reset and the evaluation is delayed.
                // There is also a maximum delay period (counted since the first event) to force
                // a rule evaluation at some point in case the events don't stop coming in
                // and resetting the minimum delay.
                let min_timeout = sleep(Duration::from_millis(PROFILE_WATCHER_MIN_DELAY_MS));
                let max_timeout = sleep(Duration::from_millis(PROFILE_WATCHER_MAX_DELAY_MS));
                tokio::pin!(min_timeout, max_timeout);

                loop {
                    select! {
                        () = &mut min_timeout => {
                            break
                        },
                        () = &mut max_timeout => {
                            trace!("profile update deadline reached");
                            break
                        },
                        Some(event) = event_rx.recv() => {
                            trace!("got another process event, delaying profile update");
                            min_timeout.as_mut().reset(tokio::time::Instant::now() + Duration::from_millis(PROFILE_WATCHER_MIN_DELAY_MS));
                            handle_profile_event(&event, &mut state, &handler);
                        }
                    }
                }

                update_profile(&state, &handler, &profile_rules).await;
            },
        }
    }

    if let Some(handle) = gamemode_task {
        handle.abort();
    }
}

fn handle_profile_event(event: &ProfileWatcherEvent, state: &mut WatcherState, handler: &Handler) {
    trace!("profile watcher event: {event:?}");

    match *event {
        ProfileWatcherEvent::Process(PEvent::Exec(pid)) => match process::get_pid_info(pid) {
            Ok(info) => {
                if info.name.as_ref() == gamemode::PROCESS_NAME {
                    info!("detected gamemode daemon, reloading profile watcher");
                    handler.start_profile_watcher();
                }

                state.process_list.insert(pid, info);
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
}

async fn update_profile(
    state: &WatcherState,
    handler: &Handler,
    profile_rules: &[(Rc<str>, ProfileRule)],
) {
    let started_at = Instant::now();
    let new_profile = evaluate_current_profile(state, profile_rules);
    trace!("evaluated profile rules in {:?}", started_at.elapsed());

    if handler.config.borrow().current_profile != new_profile {
        if let Some(name) = &new_profile {
            info!("setting current profile to {name}");
        } else {
            info!("setting default profile");
        }

        if let Err(err) = handler.set_current_profile(new_profile).await {
            error!("failed to apply profile: {err:#}");
        }
    }
}

/// Returns the new active profile
fn evaluate_current_profile(
    state: &WatcherState,
    profile_rules: &[(Rc<str>, ProfileRule)],
) -> Option<Rc<str>> {
    // TODO: fast path to re-evaluate only a single event and not the whole state?
    for pid in state.gamemode_games.iter().rev() {
        for (profile_name, rule) in profile_rules {
            if let ProfileRule::Gamemode(process_filter) = &rule {
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
        for (profile_name, rule) in profile_rules {
            if let ProfileRule::Process(rule) = &rule {
                if process_rule_matches(rule, process) {
                    return Some(profile_name.clone());
                }
            }
        }
    }

    None
}

#[inline]
fn process_rule_matches(rule: &ProcessProfileRule, process: &ProcessInfo) -> bool {
    process.name.as_ref() == rule.name
        && rule
            .args
            .as_ref()
            .map_or(true, |wanted_args| process.cmdline.contains(wanted_args))
}

#[cfg(test)]
mod tests {
    use super::{evaluate_current_profile, process::ProcessInfo, WatcherState};
    use copes::solver::PID;
    use indexmap::{IndexMap, IndexSet};
    use lact_schema::{ProcessProfileRule, ProfileRule};
    use pretty_assertions::assert_eq;
    use std::rc::Rc;

    #[test]
    fn evaluate_basic_profile() {
        let mut state = WatcherState {
            process_list: IndexMap::from([(
                PID::from(1),
                ProcessInfo {
                    name: "game1".into(),
                    cmdline: "".into(),
                },
            )]),
            gamemode_games: IndexSet::new(),
        };

        let profile_rules = vec![
            (
                "1".into(),
                ProfileRule::Process(ProcessProfileRule {
                    name: "game1".into(),
                    args: None,
                }),
            ),
            (
                "2".into(),
                ProfileRule::Process(ProcessProfileRule {
                    name: "game2".into(),
                    args: None,
                }),
            ),
        ];

        assert_eq!(
            Some(Rc::from("1")),
            evaluate_current_profile(&state, &profile_rules)
        );

        state.process_list.get_mut(&PID::from(1)).unwrap().name = "game2".into();
        assert_eq!(
            Some(Rc::from("2")),
            evaluate_current_profile(&state, &profile_rules)
        );

        state.process_list.get_mut(&PID::from(1)).unwrap().name = "game3".into();
        assert_eq!(None, evaluate_current_profile(&state, &profile_rules));
    }
}

#[cfg(feature = "bench")]
mod benches {
    use super::{evaluate_current_profile, process::ProcessInfo, WatcherState};
    use copes::solver::PID;
    use divan::Bencher;
    use indexmap::IndexSet;
    use lact_schema::{ProcessProfileRule, ProfileRule};
    use std::{hint::black_box, rc::Rc};

    #[divan::bench(sample_size = 1000, min_time = 2)]
    fn evaluate_profiles(bencher: Bencher) {
        let process_list = (1..2000)
            .map(|id| {
                let name = format!("process-{id}").into();
                let cmdline = format!("{name} arg1 arg2 --arg3").into();
                (PID::from(id), ProcessInfo { name, cmdline })
            })
            .collect();

        let state = WatcherState {
            process_list,
            gamemode_games: IndexSet::new(),
        };

        let profile_rules = vec![
            (
                "1".into(),
                ProfileRule::Process(ProcessProfileRule {
                    name: "game-abc".to_owned(),
                    args: None,
                }),
            ),
            (
                "2".into(),
                ProfileRule::Process(ProcessProfileRule {
                    name: "game-1034".to_owned(),
                    args: None,
                }),
            ),
        ];

        bencher.bench_local(move || -> Option<Rc<str>> {
            evaluate_current_profile(black_box(&state), black_box(&profile_rules))
        });
    }
}
