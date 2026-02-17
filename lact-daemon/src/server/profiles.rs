mod gamemode;
mod process;

use crate::server::{handler::Handler, profiles::gamemode::GameModeConnector};
use lact_schema::{ProfileRule, ProfileWatcherState};
use libcopes::prevent;
use std::{
    rc::Rc,
    time::{Duration, Instant},
};
use tokio::{
    runtime, select,
    sync::{Mutex, Notify, mpsc},
    time::sleep,
};
use tracing::{debug, error, info, trace};

static PROFILE_WATCHER_LOCK: Mutex<()> = Mutex::const_new(());

const PROFILE_WATCHER_MIN_DELAY_MS: u64 = 50;
const PROFILE_WATCHER_MAX_DELAY_MS: u64 = 500;

#[derive(Debug)]
enum ProfileWatcherEvent {
    Process(prevent),
    Gamemode(prevent),
}

pub enum ProfileWatcherCommand {
    Stop,
    /// Manually force a re-evaluation of the rules, such as when the rules were edited
    Update,
}

pub async fn run_watcher(handler: Handler, mut command_rx: mpsc::Receiver<ProfileWatcherCommand>) {
    debug!(
        "starting new watcher task (total task count: {})",
        runtime::Handle::current().metrics().num_alive_tasks()
    );

    let _guard = PROFILE_WATCHER_LOCK.lock().await;

    let mut state = ProfileWatcherState::default();
    process::load_full_process_list(&mut state);
    info!("loaded {} processes", state.process_list.len());

    let (event_tx, mut event_rx) = mpsc::channel(128);

    process::start_listener(event_tx.clone());

    let gamemode_stop_notify = Rc::new(Notify::new());
    let mut gamemode_task = None;
    if let Some(gamemode) = GameModeConnector::new(&state.process_list) {
        match gamemode.list_games().await {
            Ok(games) => {
                info!(
                    "connected to gamemode, games currently running: {}",
                    games.len()
                );
                for pid in games {
                    state.gamemode_games.insert(pid);
                }
            }
            Err(err) => {
                error!("could not list gamemode games: {err}");
            }
        }

        match gamemode.receieve_events(&gamemode_stop_notify) {
            Ok(mut rx) => {
                let event_tx = event_tx.clone();
                let stop_notify = gamemode_stop_notify.clone();

                let handle = tokio::task::spawn_local(async move {
                    loop {
                        select! {
                            Some(event) = rx.recv() => {
                                debug!("gamemode event: {event}");
                                let _ = event_tx.send(ProfileWatcherEvent::Gamemode(event)).await;
                            },
                            () = stop_notify.notified() => {
                                break;
                            }
                        };
                    }
                    debug!("exited gamemode watcher");
                });
                gamemode_task = Some(handle);
            }
            Err(err) => {
                error!("Could not get gamemode event stream: {err:#}");
            }
        }
    }

    *handler.profile_watcher_state.borrow_mut() = Some(state);

    update_profile(&handler).await;

    let mut should_reload = false;

    loop {
        select! {
            Some(cmd) = command_rx.recv() => {
                match cmd {
                    ProfileWatcherCommand::Stop => break,
                    ProfileWatcherCommand::Update => {
                        update_profile(&handler).await;
                    }
                }
            }
            Some(event) = event_rx.recv() => {
                handle_profile_event(&event, &handler, &mut should_reload);

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
                            handle_profile_event(&event, &handler, &mut should_reload);
                        }
                    }
                }

                update_profile(&handler).await;
            }
        }

        if should_reload {
            let handler = handler.clone();
            tokio::task::spawn_local(async move { handler.start_profile_watcher().await });
            break;
        }
    }

    handler.profile_watcher_state.borrow_mut().take();

    if let Some(handle) = gamemode_task {
        gamemode_stop_notify.notify_waiters();
        handle.await.unwrap();
    }

    debug!("profile watched exited");
}

fn handle_profile_event(event: &ProfileWatcherEvent, handler: &Handler, should_reload: &mut bool) {
    let mut state_guard = handler.profile_watcher_state.borrow_mut();
    let Some(state) = state_guard.as_mut() else {
        return;
    };

    match *event {
        ProfileWatcherEvent::Process(prevent::Exec(pid)) => match process::get_pid_info(pid) {
            Ok(info) => {
                trace!("process {pid} ({}) started", info.name);
                if info.name.as_ref() == gamemode::PROCESS_NAME {
                    info!("detected gamemode daemon, reloading profile watcher");
                    *should_reload = true;
                }
                state.push_process(*pid.as_ref(), info);
            }
            Err(err) => {
                debug!("could not get info for process {pid}: {err}");
            }
        },
        ProfileWatcherEvent::Process(prevent::Exit(pid)) => {
            if let Some(info) = state.remove_process(*pid.as_ref()) {
                trace!("process {pid} ({}) exited", info.name);
            } else {
                trace!("process {pid} exited");
            }
        }
        ProfileWatcherEvent::Gamemode(prevent::Exec(pid)) => {
            state.gamemode_games.insert(*pid.as_ref());
        }
        ProfileWatcherEvent::Gamemode(prevent::Exit(pid)) => {
            state.gamemode_games.shift_remove(pid.as_ref());
        }
    }
}

async fn update_profile(handler: &Handler) {
    let new_profile = {
        let config = handler.config.read().await;
        let profile_rules = config
            .profiles
            .iter()
            .filter_map(|(name, profile)| Some((name, profile.rule.as_ref()?)));

        let state_guard = handler.profile_watcher_state.borrow();
        if let Some(state) = state_guard.as_ref() {
            let started_at = Instant::now();
            let new_profile = evaluate_current_profile(state, profile_rules);
            trace!("evaluated profile rules in {:?}", started_at.elapsed());
            new_profile.cloned()
        } else {
            None
        }
    };

    if handler.config.read().await.current_profile != new_profile {
        if let Some(name) = &new_profile {
            info!("setting current profile to '{name}'");
        } else {
            info!("setting default profile");
        }

        if let Err(err) = handler.set_current_profile(new_profile).await {
            error!("failed to apply profile: {err:#}");
        }
    }
}

/// Returns the new active profile
fn evaluate_current_profile<'a>(
    state: &ProfileWatcherState,
    profile_rules: impl Iterator<Item = (&'a Rc<str>, &'a ProfileRule)>,
) -> Option<&'a Rc<str>> {
    for (profile_name, rule) in profile_rules {
        if profile_rule_matches(state, rule) {
            return Some(profile_name);
        }
    }

    None
}

#[inline]
pub(crate) fn profile_rule_matches(state: &ProfileWatcherState, rule: &ProfileRule) -> bool {
    match rule {
        ProfileRule::Process(process_rule) => {
            if let Some(pids) = state.process_names_map.get(&process_rule.name) {
                match &process_rule.args {
                    Some(args_filter) => {
                        for pid in pids {
                            if let Some(process_info) = state.process_list.get(pid) {
                                if process_info.cmdline.contains(args_filter) {
                                    return true;
                                }
                            } else {
                                error!("process {pid} not found in process map");
                            }
                        }
                    }
                    None => return true,
                }
            }
        }
        ProfileRule::Gamemode(None) => return !state.gamemode_games.is_empty(),
        ProfileRule::Gamemode(Some(gamemode_rule)) => {
            if let Some(pids) = state.process_names_map.get(&gamemode_rule.name) {
                for pid in pids {
                    if state.gamemode_games.contains(pid) {
                        match &gamemode_rule.args {
                            Some(args_filter) => {
                                if let Some(process_info) = state.process_list.get(pid) {
                                    if process_info.cmdline.contains(args_filter) {
                                        return true;
                                    }
                                } else {
                                    error!("process {pid} not found in process map");
                                }
                            }
                            None => return true,
                        }
                    }
                }
            }
        }
        ProfileRule::And(rules) => {
            return !rules.is_empty() && rules.iter().all(|rule| profile_rule_matches(state, rule));
        }
        ProfileRule::Or(rules) => {
            return !rules.is_empty() && rules.iter().any(|rule| profile_rule_matches(state, rule));
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::evaluate_current_profile;
    use lact_schema::{ProcessProfileRule, ProfileProcessInfo, ProfileRule, ProfileWatcherState};
    use pretty_assertions::assert_eq;
    use std::rc::Rc;

    #[test]
    fn evaluate_basic_profile() {
        let mut state = ProfileWatcherState::default();
        state.push_process(
            1,
            ProfileProcessInfo {
                name: "game1".into(),
                cmdline: "".into(),
            },
        );

        let profile_rules = [
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
            Some(&Rc::from("1")),
            evaluate_current_profile(&state, profile_rules.iter().map(|(key, rule)| (key, rule)))
        );

        state.push_process(
            1,
            ProfileProcessInfo {
                name: "game2".into(),
                cmdline: "".into(),
            },
        );
        assert_eq!(
            Some(&Rc::from("2")),
            evaluate_current_profile(&state, profile_rules.iter().map(|(key, rule)| (key, rule)))
        );

        state.push_process(
            1,
            ProfileProcessInfo {
                name: "game3".into(),
                cmdline: "".into(),
            },
        );
        assert_eq!(
            None,
            evaluate_current_profile(&state, profile_rules.iter().map(|(key, rule)| (key, rule)))
        );
    }
}

#[cfg(feature = "bench")]
mod benches {
    use super::evaluate_current_profile;
    use divan::Bencher;
    use lact_schema::{ProcessProfileRule, ProfileProcessInfo, ProfileRule, ProfileWatcherState};
    use std::hint::black_box;

    #[divan::bench(sample_size = 1000, min_time = 2)]
    fn evaluate_profiles(bencher: Bencher) {
        let mut state = ProfileWatcherState::default();

        for pid in 1..2000 {
            let name = format!("process-{pid}").into();
            let cmdline = format!("{name} arg1 arg2 --arg3").into();
            state.push_process(pid, ProfileProcessInfo { name, cmdline });
        }

        let profile_rules = [
            (
                "1".into(),
                ProfileRule::Process(ProcessProfileRule {
                    name: "game-abc".into(),
                    args: None,
                }),
            ),
            (
                "2".into(),
                ProfileRule::Process(ProcessProfileRule {
                    name: "game-1034".into(),
                    args: None,
                }),
            ),
        ];

        bencher.bench_local(move || {
            evaluate_current_profile(
                black_box(&state),
                black_box(profile_rules.iter().map(|(key, rule)| (key, rule))),
            );
        });
    }
}
