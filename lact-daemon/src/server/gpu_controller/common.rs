pub mod fan_control;
pub mod fdinfo;

use libcopes::PID;
use std::io;
use tracing::debug;

pub fn resolve_process_name(pid: PID) -> io::Result<(String, String)> {
    let cmdline = libcopes::io::cmdline_reader(pid)?;

    let name = match libcopes::io::exe_reader(pid) {
        Ok(exe) => libcopes::get_process_executed_file(exe, &cmdline).to_string(),
        Err(err) => {
            debug!("could not fetch exe for {pid}: {err}");
            let args = cmdline.as_ref();
            args.first().and_then(|arg| arg.to_str()).map_or_else(
                || "<Unknown>".to_owned(),
                |arg| arg.split('/').last().unwrap_or(arg).to_owned(),
            )
        }
    };

    let args = cmdline
        .as_ref()
        .iter()
        .skip(1)
        .map(|part| part.to_string_lossy())
        .collect::<Vec<_>>()
        .join(" ");

    Ok((name, args))
}
