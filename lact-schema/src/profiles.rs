use crate::{ProcessInfo, ProfileWatcherState};
use std::{collections::hash_map::Entry, fmt};

impl fmt::Debug for ProfileWatcherState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProfileWatcherState")
            .field("process_list", &self.process_list.len())
            .field("gamemode_games", &self.gamemode_games.len())
            .field("process_names_map", &self.process_names_map.len())
            .finish()
    }
}

impl ProfileWatcherState {
    pub fn push_process(&mut self, pid: i32, info: ProcessInfo) {
        let name = info.name.clone();

        if let Some(old_info) = self.process_list.insert(pid, info) {
            // In case we replaced a process with the same PID (this should normally never happen, but maybe we missed an exit event?)
            // the old name needs to be dropped as well.
            if let Entry::Occupied(mut entry) = self.process_names_map.entry(old_info.name) {
                entry.get_mut().remove(&pid);
                if entry.get().is_empty() {
                    entry.remove();
                }
            }
        }

        self.process_names_map.entry(name).or_default().insert(pid);
    }

    pub fn remove_process(&mut self, pid: i32) -> Option<ProcessInfo> {
        if let Some(mut info) = self.process_list.shift_remove(&pid) {
            if let Entry::Occupied(mut entry) = self.process_names_map.entry(info.name) {
                entry.get_mut().remove(&pid);
                if entry.get().is_empty() {
                    let (key, _) = entry.remove_entry();
                    info.name = key;
                    return Some(info);
                }
            }
        }
        None
    }
}
