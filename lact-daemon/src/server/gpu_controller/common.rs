pub mod fan_control;
pub mod fdinfo;

use libcopes::PID;
use std::io;

pub fn resolve_process_name(pid: PID) -> io::Result<(String, String)> {
    let exe = libcopes::io::exe_reader(pid)?;
    let cmdline = libcopes::io::cmdline_reader(pid)?;
    let name = libcopes::get_process_executed_file(exe, &cmdline).to_string();
    let args = cmdline
        .as_ref()
        .iter()
        .skip(1)
        .map(|part| part.to_string_lossy())
        .collect::<Vec<_>>()
        .join(" ");

    Ok((name, args))
}
