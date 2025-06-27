use anyhow::Context;
use lact_schema::{ProcessInfo, ProcessList, ProcessType, ProcessUtilizationType};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
    time::Instant,
};
use tracing::error;

use crate::server::gpu_controller::{common::resolve_process_name, CommonControllerInfo};

pub struct DrmUtilMap {
    timestamp: Instant,
    pids: HashMap<u32, HashMap<ProcessUtilizationType, u64>>,
}

pub fn read_process_list(
    controller_info: &CommonControllerInfo,
    vram_keys: &[&str],
    engines: EngineUtilTypes,
    last_total_time_map: &mut Option<DrmUtilMap>,
) -> anyhow::Result<ProcessList> {
    let dri_paths = [
        controller_info.get_drm_card()?,
        controller_info.get_drm_render()?,
    ];

    let procs = fs::read_dir("/proc").context("could not read /proc: {err}")?;

    let mut processes = BTreeMap::new();
    let mut supported_util_types = HashSet::new();

    let timestamp = Instant::now();

    let mut total_time_map =
        HashMap::with_capacity(last_total_time_map.as_ref().map_or(0, |map| map.pids.len()));

    for entry in procs {
        let entry = entry?;
        if let Some(pid) = entry
            .file_name()
            .to_str()
            .and_then(|name| name.parse::<u32>().ok())
        {
            if pid == 1 {
                continue;
            }

            match collect_proc_util(&entry.path(), &dri_paths, vram_keys, engines) {
                Ok(utils) => {
                    let mut pid_total_time: HashMap<ProcessUtilizationType, u64> = HashMap::new();
                    let mut processed_client_ids = HashSet::new();
                    let mut memory_used = 0;
                    let mut anything_used = false;

                    for util in utils {
                        if processed_client_ids.insert(util.client_id) {
                            for (util_type, total_time) in util.total_time {
                                supported_util_types.insert(util_type);

                                *pid_total_time.entry(util_type).or_default() += total_time;
                                anything_used |= total_time > 0;
                            }
                            memory_used += util.memory_used;
                        }
                    }

                    let mut process_util = HashMap::with_capacity(pid_total_time.len());

                    if let Some(last_total_time_map) = last_total_time_map {
                        if let Some(last_pid_util) = last_total_time_map.pids.get(&pid) {
                            let wall_time_delta =
                                (timestamp - last_total_time_map.timestamp).as_nanos();

                            for (util_type, current_util) in &pid_total_time {
                                if let Some(last_util) = last_pid_util.get(util_type) {
                                    let engine_time_delta = *current_util - *last_util;

                                    #[allow(
                                        clippy::cast_lossless,
                                        clippy::cast_possible_truncation,
                                        clippy::cast_sign_loss,
                                        clippy::cast_precision_loss
                                    )]
                                    process_util.insert(
                                        *util_type,
                                        ((engine_time_delta as f64 / wall_time_delta as f64)
                                            * 100.0) as u32,
                                    );
                                }
                            }
                        }
                    }

                    if anything_used {
                        #[allow(clippy::cast_possible_wrap)]
                        let (name, args) = resolve_process_name((pid as i32).into())
                            .unwrap_or_else(|_| ("<Unknown>".to_owned(), String::new()));

                        let mut types = vec![];

                        if pid_total_time
                            .get(&ProcessUtilizationType::Graphics)
                            .is_some_and(|value| *value > 0)
                        {
                            types.push(ProcessType::Graphics);
                        }
                        if pid_total_time
                            .get(&ProcessUtilizationType::Compute)
                            .is_some_and(|value| *value > 0)
                        {
                            types.push(ProcessType::Compute);
                        }

                        processes.insert(
                            pid,
                            ProcessInfo {
                                name,
                                args,
                                memory_used,
                                types,
                                util: process_util,
                            },
                        );
                    }

                    total_time_map.insert(pid, pid_total_time);
                }
                Err(err) => {
                    error!("could not fetch fdinfo for pid {pid}: {err:#}");
                }
            }
        }
    }

    *last_total_time_map = Some(DrmUtilMap {
        timestamp,
        pids: total_time_map,
    });

    Ok(ProcessList {
        processes,
        supported_util_types,
    })
}

fn collect_proc_util<'a>(
    pid_path: &Path,
    dri_paths: &'a [PathBuf],
    vram_keys: &'a [&str],
    engines: EngineUtilTypes,
) -> anyhow::Result<impl Iterator<Item = ProcessDrmUtil> + 'a> {
    let fdinfo_root = pid_path.join("fdinfo");
    let dir = fs::read_dir(pid_path.join("fd"))?;

    let mut fdinfo_buf = String::new();

    let iter = dir
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            if let Ok(target) = fs::read_link(entry.path()) {
                if dri_paths.contains(&target) {
                    Some(entry.file_name())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .filter_map(move |fd| {
            let fdinfo_path = fdinfo_root.join(fd);

            fdinfo_buf.clear();

            File::open(&fdinfo_path)
                .and_then(|mut file| file.read_to_string(&mut fdinfo_buf))
                .map_err(|err| {
                    error!(
                        "could not read fdinfo file at {}: {err}",
                        fdinfo_path.display()
                    );
                })
                .ok()?;

            parse_fdinfo(&fdinfo_buf, vram_keys, engines)
                .map_err(|err| {
                    error!(
                        "could not parse fdinfo at {}: {err:#}",
                        fdinfo_path.display()
                    );
                })
                .ok()
        });

    Ok(iter)
}

pub struct ProcessDrmUtil {
    pub client_id: u32,
    pub total_time: Vec<(ProcessUtilizationType, u64)>,
    pub memory_used: u64,
}

pub fn parse_fdinfo(
    data: &str,
    vram_keys: &[&str],
    engines: EngineUtilTypes,
) -> anyhow::Result<ProcessDrmUtil> {
    let items = data
        .lines()
        .filter(|line| !line.is_empty())
        .filter_map(|line| line.split_once(':'));

    let mut client_id = None;
    let mut total_time = Vec::with_capacity(engines.len());

    let mut memory_used = 0;
    let mut memory_key_idx = None;

    for (key, value) in items {
        let value = value.trim_ascii();

        match key {
            "drm-client-id" => {
                client_id = Some(value.parse().context("Could not parse client id")?);
            }
            _ => {
                if let Some(engine) = key.strip_prefix("drm-engine-") {
                    if let Some((_, util_type)) = engines.iter().find(|(name, _)| engine == *name) {
                        if let Some(time) =
                            value.strip_suffix(" ns").and_then(|time| time.parse().ok())
                        {
                            total_time.push((*util_type, time));
                        }
                    }
                } else {
                    for (i, vram_key) in vram_keys.iter().enumerate() {
                        // Prioritize first vram key in the list
                        if key == *vram_key && memory_key_idx.is_none_or(|idx| i < idx) {
                            if let Some(value) = value
                                .strip_suffix(" KiB")
                                .and_then(|value| value.parse::<u64>().ok())
                            {
                                memory_used = value * 1024;
                                memory_key_idx = Some(i);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(ProcessDrmUtil {
        client_id: client_id.context("Missing client id")?,
        total_time,
        memory_used,
    })
}

pub type EngineUtilTypes = &'static [(&'static str, ProcessUtilizationType)];
