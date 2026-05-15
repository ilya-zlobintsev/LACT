use anyhow::Context;
use std::{fs, path::Path};
use tracing::warn;

pub fn get_display_info(device_path: &Path) -> Option<()> {
    try_get_display_info(device_path)
        .inspect_err(|err| {
            warn!("could not fetch displays info: {err:#}");
        })
        .ok()
}

fn try_get_display_info(device_path: &Path) -> anyhow::Result<()> {
    let path_parent = device_path.parent().context("Invalid path")?;
    let card_entry_name = path_parent
        .file_name()
        .and_then(|name| name.to_str())
        .context("Invalid path")?;

    let entries = fs::read_dir(path_parent)?;

    for entry in entries {
        let entry = entry?;
        let display_entry_path = entry.path();

        if display_entry_path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with(card_entry_name))
        {
            let status_path = display_entry_path.join("status");

            if !status_path.exists() {
                continue;
            }

            match fs::read_to_string(&status_path) {
                Ok(status) if status.trim_ascii() == "connected" => {
                    if let Err(err) = get_display_entry(&display_entry_path) {
                        warn!(
                            "could not parse display info at {}: {err:#}",
                            display_entry_path.display()
                        );
                    }
                }
                Ok(_) => (),
                Err(err) => warn!(
                    "could not read display status from {}: {err}",
                    status_path.display()
                ),
            }
        }
    }

    Ok(())
}

fn get_display_entry(path: &Path) -> anyhow::Result<()> {
    let edid_data = fs::read(path.join("edid")).context("Could not read edid")?;
    let info =
        libdisplay_info::info::Info::parse_edid(&edid_data).context("Could not parse edid")?;

    println!(
        "{:?} {:?} {:?} {:?}",
        info.make(),
        info.serial(),
        info.model(),
        info.failure_msg()
    );

    Ok(())
}
