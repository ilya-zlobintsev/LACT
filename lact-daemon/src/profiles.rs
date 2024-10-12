mod gamemode;
mod process;
mod rule;

use crate::server::handler::Handler;
use copes::solver::{PEvent, PID};
use futures::StreamExt;
use indexmap::{IndexMap, IndexSet};
use process::ProcessInfo;
use rule::{CompiledProcessRule, CompiledRule};
use std::{rc::Rc, time::Instant};
use string_interner::{backend::StringBackend, StringInterner};
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
    let mut interner = StringInterner::default();

    let profile_rules = handler
        .config
        .borrow()
        .profiles
        .iter()
        .filter_map(|(name, profile)| {
            let rule = profile.rule.as_ref()?;
            Some((name.clone(), CompiledRule::new(rule, &mut interner)))
        })
        .collect::<Vec<_>>();

    let process_list = process::load_full_process_list(&mut interner).collect();

    let mut state = WatcherState {
        process_list,
        gamemode_games: IndexSet::new(),
    };
    info!("loaded {} processes", state.process_list.len());

    let (event_tx, mut event_rx) = mpsc::channel(128);

    process::start_listener(event_tx.clone());

    if let Some((_conn, proxy)) = gamemode::connect(&state.process_list, &interner).await {
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

    update_profile(&state, &handler, &profile_rules, &interner).await;

    loop {
        select! {
            () = handler.profile_watcher_stop_notify.notified() => break,
            Some(event) = event_rx.recv() => {
                trace!("profile watcher event: {event:?}");
                match event {
                    ProfileWatcherEvent::Process(PEvent::Exec(pid)) => match process::get_pid_info(pid, &mut interner) {
                        Ok(info) => {
                            if info.resolve_name(&interner) == gamemode::PROCESS_NAME {
                                info!("detected gamemode daemon, reloading profile watcher");
                                tokio::task::spawn_local(run_watcher(handler));
                                break;
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

                update_profile(&state, &handler, &profile_rules, &interner).await;
            },
        }
    }
}

async fn update_profile(
    state: &WatcherState,
    handler: &Handler,
    profile_rules: &[(Rc<str>, CompiledRule)],
    interner: &StringInterner<StringBackend>,
) {
    let started_at = Instant::now();
    let new_profile = evaluate_current_profile(state, profile_rules, interner);
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
fn evaluate_current_profile(
    state: &WatcherState,
    profile_rules: &[(Rc<str>, CompiledRule)],
    interner: &StringInterner<StringBackend>,
) -> Option<Rc<str>> {
    // TODO: fast path to re-evaluate only a single event and not the whole state?
    for pid in state.gamemode_games.iter().rev() {
        for (profile_name, rule) in profile_rules {
            if let CompiledRule::Gamemode(process_filter) = &rule {
                match process_filter {
                    Some(filter) => {
                        if let Some(process) = state.process_list.get(pid) {
                            if process_rule_matches(filter, process, interner) {
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
            if let CompiledRule::Process(rule) = &rule {
                if process_rule_matches(rule, process, interner) {
                    return Some(profile_name.clone());
                }
            }
        }
    }

    None
}

#[inline]
fn process_rule_matches(
    rule: &CompiledProcessRule,
    process: &ProcessInfo,
    interner: &StringInterner<StringBackend>,
) -> bool {
    process.name == rule.name
        && rule.args.as_ref().map_or(true, |wanted_args| {
            let wanted_args = interner.resolve(*wanted_args).unwrap();
            process.resolve_cmdline(interner).contains(wanted_args)
        })
}

#[cfg(test)]
mod tests {
    use super::{process::ProcessInfo, WatcherState};
    use crate::profiles::{
        evaluate_current_profile,
        rule::{CompiledProcessRule, CompiledRule},
    };
    use copes::solver::PID;
    use indexmap::{IndexMap, IndexSet};
    use pretty_assertions::assert_eq;
    use std::rc::Rc;
    use string_interner::StringInterner;

    #[test]
    fn evaluate_basic_profile() {
        let mut interner = StringInterner::default();

        let mut state = WatcherState {
            process_list: IndexMap::from([(
                PID::from(1),
                ProcessInfo {
                    name: interner.get_or_intern("game1"),
                    cmdline: interner.get_or_intern(""),
                },
            )]),
            gamemode_games: IndexSet::new(),
        };

        let profile_rules = vec![
            (
                "1".into(),
                CompiledRule::Process(CompiledProcessRule {
                    name: interner.get_or_intern("game1"),
                    args: None,
                }),
            ),
            (
                "2".into(),
                CompiledRule::Process(CompiledProcessRule {
                    name: interner.get_or_intern("game2"),
                    args: None,
                }),
            ),
        ];

        assert_eq!(
            Some(Rc::from("1")),
            evaluate_current_profile(&state, &profile_rules, &interner)
        );

        state.process_list.get_mut(&PID::from(1)).unwrap().name = interner.get_or_intern("game2");
        assert_eq!(
            Some(Rc::from("2")),
            evaluate_current_profile(&state, &profile_rules, &interner)
        );

        state.process_list.get_mut(&PID::from(1)).unwrap().name = interner.get_or_intern("game3");
        assert_eq!(
            None,
            evaluate_current_profile(&state, &profile_rules, &interner)
        );
    }
}

#[cfg(feature = "bench")]
mod benches {
    use super::{
        evaluate_current_profile,
        process::ProcessInfo,
        rule::{CompiledProcessRule, CompiledRule},
        WatcherState,
    };
    use copes::solver::PID;
    use divan::Bencher;
    use indexmap::IndexSet;
    use std::{hint::black_box, rc::Rc};
    use string_interner::StringInterner;

    #[divan::bench(sample_size = 1000, min_time = 2)]
    fn evaluate_profiles(bencher: Bencher) {
        let mut interner = StringInterner::default();

        let process_list = (1..2000)
            .map(|id| {
                let name = format!("process-{id}");
                let cmdline = format!("{name} arg1 arg2 --arg3");
                (
                    PID::from(id),
                    ProcessInfo {
                        name: interner.get_or_intern(name),
                        cmdline: interner.get_or_intern(cmdline),
                    },
                )
            })
            .collect();

        let state = WatcherState {
            process_list,
            gamemode_games: IndexSet::new(),
        };

        let profile_rules = vec![
            (
                "1".into(),
                CompiledRule::Process(CompiledProcessRule {
                    name: interner.get_or_intern("game-abc"),
                    args: None,
                }),
            ),
            (
                "2".into(),
                CompiledRule::Process(CompiledProcessRule {
                    name: interner.get_or_intern("game-1034"),
                    args: None,
                }),
            ),
        ];

        bencher.bench_local(move || -> Option<Rc<str>> {
            evaluate_current_profile(
                black_box(&state),
                black_box(&profile_rules),
                black_box(&interner),
            )
        });
    }
}
