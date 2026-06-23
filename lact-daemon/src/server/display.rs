use anyhow::Context;
use lact_schema::{
    DisplayConnector, DisplayInfo, DisplayManufactureDate, DisplayRefreshRateRange, DisplaysInfo,
};
use std::{collections::BTreeMap, fs, path::Path};
use tracing::warn;

const BASE_RATE_MULTIPLIER: u32 = 270;
const UHBR_RATE_MULTIPLIER: u32 = 10;
const UHBR_RATE_THRESHOLD: u32 = 1000;

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

    let connector_id = fs::read_to_string(path.join("connector_id"))
        .context("Could not read connector_id")?
        .trim_ascii()
        .parse()
        .context("Invalid connector_id")?;

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
        "DP" => DisplayConnector::DisplayPort {
            lanes: None,
            bandwidth: None,
            embedded: false,
        },
        "eDP" => DisplayConnector::DisplayPort {
            lanes: None,
            bandwidth: None,
            embedded: true,
        },
        "HDMI" => DisplayConnector::Hdmi,
        "DVI" => DisplayConnector::Dvi,
        "VGA" => DisplayConnector::Vga,
        _ => DisplayConnector::Other,
    };

    let info = DisplayInfo {
        model: info.model(),
        manufacturer: info.make(),
        product_code: edid.vendor_product().product,
        manufacture_date: Some(DisplayManufactureDate {
            year: edid.vendor_product().manufacture_year.try_into()?,
            week: edid.vendor_product().manufacture_week.try_into()?,
        }),
        #[allow(clippy::cast_sign_loss)]
        size: edid
            .screen_size()
            .width_cm
            .zip(edid.screen_size().height_cm)
            .map(|(width, height)| (width as u32, height as u32)),
        connector_type,
        connector_id,
        bit_depth: edid
            .video_input_digital()
            .and_then(|input| input.color_bit_depth)
            .and_then(|depth| depth.try_into().ok()),
        refresh_rate_range: get_refresh_rate_range(&edid),
    };
    Ok((connector.to_owned(), info))
}

fn get_refresh_rate_range(
    edid: &libdisplay_info::edid::Edid<'_>,
) -> Option<DisplayRefreshRateRange> {
    edid.display_descriptors()
        .iter()
        .find_map(libdisplay_info::edid::DisplayDescriptorRef::range_limits)
        .and_then(|range| {
            (range.min_vert_rate_hz > 0 && range.max_vert_rate_hz >= range.min_vert_rate_hz)
                .then_some(DisplayRefreshRateRange {
                    min_hz: range.min_vert_rate_hz.try_into().ok()?,
                    max_hz: range.max_vert_rate_hz.try_into().ok()?,
                })
        })
}

pub fn dp_rate_to_bandwidth(value: u32) -> u32 {
    // Ref: https://elixir.bootlin.com/linux/v7.0.10/source/drivers/gpu/drm/amd/display/dc/dc_dp_types.h#L41
    // Applies not only to AMD, as this is from the DP spec
    // Values are for conversion to Mbps
    if value < UHBR_RATE_THRESHOLD {
        value * BASE_RATE_MULTIPLIER
    } else {
        value * UHBR_RATE_MULTIPLIER
    }
}
