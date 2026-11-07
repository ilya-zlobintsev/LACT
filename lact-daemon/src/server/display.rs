use anyhow::Context;
use lact_schema::{DisplayConnector, DisplayInfo, DisplayManufactureDate, DisplaysInfo};
use std::{collections::BTreeMap, fs, path::Path};
use tracing::warn;

// Values are for conversion to Mbps
const UHBR_RATE_THRESHOLD: u32 = 1000;
const BASE_RATE_MULTIPLIER: u32 = 270;
const UHBR_RATE_MULTIPLIER: u32 = 10;

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
    };
    Ok((connector.to_owned(), info))
}

// Values are for conversion to Mbps
pub fn dp1_rate_to_bandwidth(value: u32) -> u32 {
    value * BASE_RATE_MULTIPLIER
}

// Values are for conversion to Mbps
pub fn dp2_rate_to_bandwidth(value: u32) -> u32 {
    value * UHBR_RATE_MULTIPLIER
}

/// Parse the contents of the amdgpu debugfs `link_settings` file, returning the
/// current lane count and per-lane bandwidth in Mbps.
///
/// The file contains the current, verified, reported and preferred settings on a
/// single line. The current settings describe the link the display is actually
/// running on, so those are the ones reported here.
///
/// Returns `None` when the current link settings are all zero: the connector is
/// then not driving a `DisplayPort` link at all. This happens for example with an
/// HDMI sink behind a passive DP++ adapter, where the GPU uses TMDS signalling
/// and there is no lane count or per-lane rate to report.
pub fn parse_dp_link_settings(contents: &str) -> anyhow::Result<Option<(u16, u32)>> {
    let mut parts = contents.split_ascii_whitespace().skip(1);

    let lanes = parts
        .next()
        .context("Missing lane count")?
        .parse::<u16>()
        .context("Invalid lane count")?;

    let rate = parts
        .next()
        .context("Missing bandwidth")?
        .strip_prefix("0x")
        .and_then(|value| u32::from_str_radix(value, 16).ok())
        .context("Invalid bandwidth value")?;

    if lanes == 0 || rate == 0 {
        return Ok(None);
    }

    // Ref: https://elixir.bootlin.com/linux/v7.0.10/source/drivers/gpu/drm/amd/display/dc/dc_dp_types.h#L41
    // AMD reports legacy DP link rates as DP spec codes, while DP2 UHBR
    // rates are reported in 10 Mbps units. UHBR10 starts at 1000.
    let bandwidth = if rate < UHBR_RATE_THRESHOLD {
        dp1_rate_to_bandwidth(rate)
    } else {
        dp2_rate_to_bandwidth(rate)
    };

    Ok(Some((lanes, bandwidth)))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Output of `cat /sys/kernel/debug/dri/<card>/DP-x/link_settings` on a
    // trained HBR3 link.
    const TRAINED_LINK: &str = "Current:  4  0x14  0  Verified:  4  0x14  16  Reported:  4  0x14  16  Preferred:  0  0x0  0\n";
    // A DP++ (TMDS) connection, e.g. an HDMI monitor behind a passive adapter.
    const UNTRAINED_LINK: &str =
        "Current:  0  0x0  0  Verified:  0  0x0  0  Reported:  0  0x0  0  Preferred:  0  0x0  0\n";

    #[test]
    fn parses_current_link_settings() {
        assert_eq!(
            parse_dp_link_settings(TRAINED_LINK).unwrap(),
            Some((4, 5400))
        );
    }

    #[test]
    fn parses_uhbr_link_settings() {
        let contents = "Current:  4  0x7d0  0  Verified:  4  0x7d0  0  Reported:  4  0x7d0  0  Preferred:  0  0x0  0\n";
        assert_eq!(parse_dp_link_settings(contents).unwrap(), Some((4, 20000)));
    }

    #[test]
    fn zero_link_settings_are_reported_as_unknown() {
        assert_eq!(parse_dp_link_settings(UNTRAINED_LINK).unwrap(), None);
    }

    #[test]
    fn rejects_invalid_link_settings() {
        assert!(parse_dp_link_settings("garbage").is_err());
    }
}
