use super::DriverHandle;
use anyhow::{Context, anyhow, ensure};
use nvml_wrapper::Device;

// Private layouts compared against NvAPI and GSP from R595, R610 and R615.
// These are the native RM payloads, without NvAPI's 0x10-byte transport prefix.
const GET_INFO: u32 = 0x2080_a630;
const GET_CONTROL: u32 = 0x2080_a632;
const SET_CONTROL: u32 = 0x2080_e633;
const ORDINARY_CLIENT: u8 = 0xfe;
const LOWER_LIMIT_MW: u32 = 30_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PowerLimitLayout {
    info_size: usize,
    control_size: usize,
    info_min_at: usize,
    request_at: usize,
    client_at: usize,
    mask_end: usize,
}

const EXTENDED_LAYOUT: PowerLimitLayout = PowerLimitLayout {
    info_size: 0x924,
    control_size: 0x328,
    info_min_at: 0x28,
    request_at: 0x2c,
    client_at: 0x30,
    mask_end: 0x24,
};
const LEGACY_LAYOUT: PowerLimitLayout = PowerLimitLayout {
    info_size: 0x488,
    control_size: 0x188,
    info_min_at: 0xc,
    request_at: 0xc,
    client_at: 0x10,
    mask_end: 0x8,
};

// Keep units explicit: NVML/RM use milliwatts, while config and UI use watts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::struct_field_names)]
struct PowerLimitBounds {
    min_mw: u32,
    default_mw: u32,
    max_mw: u32,
}

/// Applies the power cap through the ordinary RM power client, which also
/// accepts values below the VBIOS minimum. No cap means the default one.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn apply(
    driver: Option<&DriverHandle>,
    device: &Device<'_>,
    power_cap: Option<f64>,
) -> anyhow::Result<()> {
    let driver = driver.context("NVIDIA RM is unavailable")?;
    let limits = device
        .power_management_limit_constraints()
        .context("Could not get power cap constraints")?;
    let bounds = PowerLimitBounds {
        min_mw: limits.min_limit,
        default_mw: device
            .power_management_limit_default()
            .context("Could not get default power cap")?,
        max_mw: limits.max_limit,
    };
    let current = device
        .power_management_limit()
        .context("Could not get current cap")?;
    let limit = power_cap.map_or(bounds.default_mw, |cap| (cap * 1000.0) as u32);

    apply_with(bounds, current, limit, |cmd, data| unsafe {
        driver.query_rm_control_sized(cmd, data)
    })
    .context("Could not set power cap using ioctl")
}

/// NVML only rejects an out-of-range cap when it has to change the limit, which
/// it does not after the ioctl already applied a cap below the VBIOS minimum.
pub fn ensure_native_range(device: &Device<'_>, power_cap: Option<f64>) -> anyhow::Result<()> {
    if let Some(cap) = power_cap {
        let limits = device
            .power_management_limit_constraints()
            .context("Could not get power cap constraints")?;
        let min = f64::from(limits.min_limit) / 1000.0;
        let max = f64::from(limits.max_limit) / 1000.0;
        ensure!(
            (min..=max).contains(&cap),
            "Power cap must be between {min} and {max} W"
        );
    }
    Ok(())
}

fn apply_with(
    nvml_bounds: PowerLimitBounds,
    nvml_current_mw: u32,
    limit_mw: u32,
    mut query: impl FnMut(u32, &mut [u8]) -> anyhow::Result<()>,
) -> anyhow::Result<()> {
    // NaN, negative and oversized caps saturate to 0 or u32::MAX, both outside.
    ensure!(
        (nvml_bounds.min_mw.min(LOWER_LIMIT_MW)..=nvml_bounds.max_mw).contains(&limit_mw),
        "Power limit is outside the supported range"
    );

    let (layout, before) = probe(nvml_bounds, nvml_current_mw, &mut query)?;
    if read_u32(&before, layout.request_at) == limit_mw {
        return Ok(());
    }

    // Keep the entire current payload, changing only entry 0's request. Mask 1
    // and selector FE prevent modifying any other entry or the additional F8 client.
    let mut expected = before.clone();
    expected[layout.request_at..layout.request_at + 4].copy_from_slice(&limit_mw.to_le_bytes());
    let applied = (|| -> anyhow::Result<()> {
        let mut request = expected.clone();
        query(SET_CONTROL, &mut request).context("Could not set ordinary power request")?;
        ensure!(
            read_control(layout, &mut query)? == expected,
            "Power request readback differs"
        );
        Ok(())
    })();

    if let Err(apply_error) = applied {
        // A failed SET can have side effects. Restore even on transport failure,
        // and use FE so a previous limit below VBIOS minimum can also be restored.
        let restored = (|| -> anyhow::Result<()> {
            let mut restore = before.clone();
            query(SET_CONTROL, &mut restore)?;
            ensure!(
                read_control(layout, &mut query)? == before,
                "Restored power request differs"
            );
            Ok(())
        })();
        return match restored {
            Ok(()) => Err(apply_error.context("Previous power request restored")),
            Err(restore_error) => Err(anyhow!(
                "Power request failed: {apply_error:#}; restoration also failed: {restore_error:#}"
            )),
        };
    }
    Ok(())
}

/// Finds the layout whose bounds and ordinary request agree with NVML, using
/// only GETs. A version number is not evidence that the payload still has the
/// same layout or units.
fn probe(
    nvml_bounds: PowerLimitBounds,
    nvml_current_mw: u32,
    query: &mut impl FnMut(u32, &mut [u8]) -> anyhow::Result<()>,
) -> anyhow::Result<(PowerLimitLayout, Vec<u8>)> {
    ensure!(cfg!(target_endian = "little"), "Unsupported byte order");
    let mut errors = Vec::new();
    for layout in [EXTENDED_LAYOUT, LEGACY_LAYOUT] {
        let candidate = (|| -> anyhow::Result<Vec<u8>> {
            ensure!(
                read_bounds(layout, query)? == nvml_bounds,
                "RM power bounds differ from NVML"
            );
            let control = read_control(layout, query)?;
            ensure!(
                read_u32(&control, layout.request_at) == nvml_current_mw,
                "RM ordinary power request differs from NVML"
            );
            Ok(control)
        })();
        match candidate {
            Ok(control) => return Ok((layout, control)),
            Err(error) => errors.push(format!("{layout:?}: {error:#}")),
        }
    }
    Err(anyhow!(
        "No compatible RM power layout: {}",
        errors.join("; ")
    ))
}

fn read_bounds(
    layout: PowerLimitLayout,
    query: &mut impl FnMut(u32, &mut [u8]) -> anyhow::Result<()>,
) -> anyhow::Result<PowerLimitBounds> {
    let mut info = vec![0; layout.info_size];
    query(GET_INFO, &mut info)?;
    validate_header(layout, &info)?;
    let bounds = PowerLimitBounds {
        min_mw: read_u32(&info, layout.info_min_at),
        default_mw: read_u32(&info, layout.info_min_at + 4),
        max_mw: read_u32(&info, layout.info_min_at + 8),
    };
    ensure!(
        bounds.min_mw > 0
            && bounds.min_mw <= bounds.default_mw
            && bounds.default_mw <= bounds.max_mw,
        "Invalid RM power limit bounds"
    );
    Ok(bounds)
}

fn read_control(
    layout: PowerLimitLayout,
    query: &mut impl FnMut(u32, &mut [u8]) -> anyhow::Result<()>,
) -> anyhow::Result<Vec<u8>> {
    let mut control = vec![0; layout.control_size];
    control[4..8].copy_from_slice(&1u32.to_le_bytes());
    control[layout.client_at] = ORDINARY_CLIENT;
    query(GET_CONTROL, &mut control)?;
    validate_header(layout, &control)?;
    ensure!(
        control[layout.client_at] == ORDINARY_CLIENT,
        "Unexpected power client"
    );
    ensure!(
        !matches!(read_u32(&control, layout.request_at), 0 | u32::MAX),
        "No ordinary power request available"
    );
    Ok(control)
}

fn validate_header(layout: PowerLimitLayout, data: &[u8]) -> anyhow::Result<()> {
    ensure!(
        read_u32(data, 0) == 0xff
            && read_u32(data, 4) == 1
            && data[8..layout.mask_end].iter().all(|byte| *byte == 0),
        "Unrecognized RM power client layout"
    );
    Ok(())
}

fn read_u32(data: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    const BOUNDS: PowerLimitBounds = PowerLimitBounds {
        min_mw: 250_000,
        default_mw: 300_000,
        max_mw: 325_000,
    };

    struct FakeRm {
        layout: PowerLimitLayout,
        control: Vec<u8>,
        reads: Vec<(u32, usize)>,
        writes: Vec<Vec<u8>>,
        fail_first_write: bool,
        fail_readback: bool,
        fail_restore: bool,
    }

    impl FakeRm {
        fn new(layout: PowerLimitLayout, current: u32) -> Self {
            let mut control = vec![0; layout.control_size];
            control[..8].copy_from_slice(&[0xff, 0, 0, 0, 1, 0, 0, 0]);
            control[layout.request_at - 4..layout.request_at].copy_from_slice(&[0x67, 0x67, 0, 0]);
            control[layout.request_at..layout.request_at + 4]
                .copy_from_slice(&current.to_le_bytes());
            control[layout.client_at] = 0xfe;
            Self {
                layout,
                control,
                reads: Vec::new(),
                writes: Vec::new(),
                fail_first_write: false,
                fail_readback: false,
                fail_restore: false,
            }
        }

        fn apply(&mut self, current: u32, limit: u32) -> anyhow::Result<()> {
            apply_with(BOUNDS, current, limit, |cmd, data| self.query(cmd, data))
        }

        fn query(&mut self, cmd: u32, data: &mut [u8]) -> anyhow::Result<()> {
            match cmd {
                GET_INFO => {
                    self.reads.push((cmd, data.len()));
                    ensure!(data.len() == self.layout.info_size, "Unsupported INFO size");
                    data[..8].copy_from_slice(&[0xff, 0, 0, 0, 1, 0, 0, 0]);
                    for (index, value) in [250_000u32, 300_000, 325_000].into_iter().enumerate() {
                        let offset = self.layout.info_min_at + 4 * index;
                        data[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
                    }
                }
                GET_CONTROL => {
                    self.reads.push((cmd, data.len()));
                    ensure!(
                        data.len() == self.layout.control_size,
                        "Unsupported CONTROL size"
                    );
                    assert_eq!(data[self.layout.client_at], 0xfe);
                    if self.fail_readback && self.writes.len() == 1 {
                        anyhow::bail!("readback unavailable");
                    }
                    data.copy_from_slice(&self.control);
                }
                SET_CONTROL => {
                    assert_eq!(data.len(), self.layout.control_size);
                    assert_eq!(&data[4..8], &[1, 0, 0, 0]);
                    assert_eq!(data[self.layout.client_at], 0xfe);
                    assert!(
                        data.iter()
                            .zip(&self.control)
                            .enumerate()
                            .all(|(i, (a, b))| {
                                (self.layout.request_at..self.layout.request_at + 4).contains(&i)
                                    || a == b
                            })
                    );
                    self.writes.push(data.to_vec());
                    if self.fail_restore && self.writes.len() > 1 {
                        anyhow::bail!("restore unavailable");
                    }
                    self.control.copy_from_slice(data);
                    if self.fail_first_write && self.writes.len() == 1 {
                        anyhow::bail!("SET failed after modifying hardware");
                    }
                }
                _ => panic!("unexpected command {cmd:x}"),
            }
            Ok(())
        }
    }

    #[test]
    fn detects_both_layouts_and_changes_only_the_fe_request() {
        for layout in [EXTENDED_LAYOUT, LEGACY_LAYOUT] {
            let mut rm = FakeRm::new(layout, 250_000);
            rm.apply(250_000, 150_000).unwrap();
            let probe = if layout == EXTENDED_LAYOUT {
                vec![(GET_INFO, 0x924), (GET_CONTROL, 0x328)]
            } else {
                vec![(GET_INFO, 0x924), (GET_INFO, 0x488), (GET_CONTROL, 0x188)]
            };
            assert_eq!(rm.reads[..probe.len()], probe);
            assert_eq!(read_u32(&rm.control, layout.request_at), 150_000);

            for (current, limit) in [(150_000, 30_000), (30_000, 325_000), (325_000, 250_000)] {
                rm.apply(current, limit).unwrap();
                assert_eq!(read_u32(&rm.control, layout.request_at), limit);
            }
            // An unchanged request is not written again.
            let writes = rm.writes.len();
            rm.apply(250_000, 250_000).unwrap();
            assert_eq!(rm.writes.len(), writes);
        }
    }

    #[test]
    fn out_of_range_unknown_layouts_and_nvml_mismatches_never_write() {
        for limit in [0, 29_999, 325_001, u32::MAX] {
            let mut rm = FakeRm::new(EXTENDED_LAYOUT, 250_000);
            assert!(rm.apply(250_000, limit).is_err());
            assert!(rm.reads.is_empty() && rm.writes.is_empty());
        }

        let mut calls = Vec::new();
        assert!(
            apply_with(BOUNDS, 250_000, 150_000, |cmd, data| {
                calls.push((cmd, data.len()));
                anyhow::bail!("Unsupported payload")
            })
            .is_err()
        );
        assert_eq!(calls, [(GET_INFO, 0x924), (GET_INFO, 0x488)]);

        // Zero bounds would otherwise let a 0 W cap through the range check.
        let zero = PowerLimitBounds {
            min_mw: 0,
            default_mw: 0,
            max_mw: 0,
        };
        let valid_header_zero_bounds = |cmd, data: &mut [u8]| {
            assert_eq!(cmd, GET_INFO);
            data[..8].copy_from_slice(&[0xff, 0, 0, 0, 1, 0, 0, 0]);
            Ok(())
        };
        assert!(apply_with(zero, 250_000, 0, valid_header_zero_bounds).is_err());

        for layout in [EXTENDED_LAYOUT, LEGACY_LAYOUT] {
            let mut rm = FakeRm::new(layout, 250_000);
            assert!(rm.apply(300_000, 150_000).is_err());
            let other_bounds = PowerLimitBounds {
                max_mw: 350_000,
                ..BOUNDS
            };
            assert!(apply_with(other_bounds, 250_000, 150_000, |c, d| rm.query(c, d)).is_err());
            assert!(rm.writes.is_empty());
        }
    }

    #[test]
    fn rejects_unrecognized_headers_masks_and_client_values() {
        for layout in [EXTENDED_LAYOUT, LEGACY_LAYOUT] {
            for (at, value) in [(0, 0), (4, 3), (layout.client_at, 0xf8)] {
                let mut rm = FakeRm::new(layout, 250_000);
                rm.control[at] = value;
                assert!(rm.apply(250_000, 150_000).is_err());
                assert!(rm.writes.is_empty());
            }
            for current in [0, u32::MAX] {
                let mut rm = FakeRm::new(layout, current);
                assert!(rm.apply(current, 150_000).is_err());
                assert!(rm.writes.is_empty());
            }
        }
        // The larger group has additional mask words. Accepting only its low
        // word would allow an unexpected client to be included in a later SET.
        let mut rm = FakeRm::new(EXTENDED_LAYOUT, 250_000);
        rm.control[8] = 1;
        assert!(rm.apply(250_000, 150_000).is_err());
        assert!(rm.writes.is_empty());
    }

    #[test]
    fn restores_the_previous_request_after_a_failed_set_or_readback() {
        for layout in [EXTENDED_LAYOUT, LEGACY_LAYOUT] {
            for fail_set in [false, true] {
                let mut rm = FakeRm::new(layout, 100_000);
                let original = rm.control.clone();
                rm.fail_first_write = fail_set;
                rm.fail_readback = !fail_set;
                assert!(rm.apply(100_000, 150_000).is_err());
                assert_eq!(rm.writes.len(), 2);
                assert_eq!(rm.control, original);
            }

            let mut rm = FakeRm::new(layout, 100_000);
            rm.fail_first_write = true;
            rm.fail_restore = true;
            let error = rm.apply(100_000, 150_000).unwrap_err();
            assert!(error.to_string().contains("restoration also failed"));
        }
    }
}
