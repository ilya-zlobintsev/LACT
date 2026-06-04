use anyhow::Context;
use lact_schema::{DisplayConnector, DisplayInfo, DisplaysInfo};
use std::{collections::BTreeMap, fs, path::Path};
use tracing::warn;

pub fn get_base_displays_info(device_path: &Path) -> anyhow::Result<DisplaysInfo> {
    let path_parent = device_path.parent().context("Invalid path")?;
    let card_entry_name = path_parent
        .file_name()
        .and_then(|name| name.to_str())
        .context("Invalid path")?;

    let entries = fs::read_dir(path_parent)?;

    let mut displays = BTreeMap::new();

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
                    match get_display_entry(&display_entry_path) {
                        Ok((key, info)) => {
                            displays.insert(key, info);
                        }
                        Err(err) => {
                            warn!(
                                "could not parse display info at {}: {err:#}",
                                display_entry_path.display()
                            );
                        }
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

    Ok(DisplaysInfo { displays })
}

fn get_display_entry(path: &Path) -> anyhow::Result<(String, DisplayInfo)> {
    let edid_data = fs::read(path.join("edid")).context("Could not read edid")?;
    let info =
        libdisplay_info::info::Info::parse_edid(&edid_data).context("Could not parse edid")?;
    let edid = info.edid().context("Missing edid in parsed info")?;

    let (_, connector) = path
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(|part| part.split_once('-'))
        .with_context(|| format!("Unexpected display connector path {}", path.display()))?;

    let connector_type = match connector
        .split_once('-')
        .context("Unexpected connector name")?
        .0
    {
        "DP" | "eDP" => DisplayConnector::DisplayPort { lanes: 0, rate: 0 },
        "HDMI" => DisplayConnector::Hdmi,
        "DVI" => DisplayConnector::Dvi,
        "VGA" => DisplayConnector::Vga,
        _ => DisplayConnector::Other,
    };

    let info = DisplayInfo {
        model: info.model(),
        manufacturer: info.make(),
        product_code: edid.vendor_product().product,
        manufacture_year: edid.vendor_product().manufacture_year as u16,
        manufacture_week: edid.vendor_product().manufacture_week as u8,
        size: edid
            .screen_size()
            .width_cm
            .zip(edid.screen_size().height_cm)
            .map(|(width, height)| (width as u32, height as u32)),
        connector_type,
    };
    Ok((connector.to_owned(), info))
}
