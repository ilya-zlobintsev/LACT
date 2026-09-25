use super::DriverHandle;
use anyhow::{Context, anyhow, ensure};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// Keep units explicit: NVML/RM use milliwatts, while config and UI use watts.
#[allow(clippy::struct_field_names)]
pub struct PowerLimitBounds {
    pub min_mw: u32,
    pub default_mw: u32,
    pub max_mw: u32,
}

impl PowerLimitBounds {
    pub fn lower_min_mw(self) -> u32 {
        self.min_mw.min(LOWER_LIMIT_MW)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LowerPowerLimit {
    pub bounds: PowerLimitBounds,
    layout: PowerLimitLayout,
}

impl LowerPowerLimit {
    pub fn lower_min_mw(self) -> u32 {
        self.bounds.lower_min_mw()
    }
}

impl DriverHandle {
    /// Only advertise the private route when its layout, units and GPU identity
    /// agree with NVML. Discovery does not issue SET, including on startup.
    pub fn probe_lower_power_limit(
        &self,
        nvml_bounds: PowerLimitBounds,
        nvml_current_mw: u32,
    ) -> anyhow::Result<Option<LowerPowerLimit>> {
        probe(nvml_bounds, nvml_current_mw, |cmd, data| unsafe {
            self.query_rm_control_sized(cmd, data)
        })
    }

    pub fn set_lower_power_limit(
        &self,
        limit_mw: u32,
        support: LowerPowerLimit,
    ) -> anyhow::Result<()> {
        set_limit(limit_mw, support, |cmd, data| unsafe {
            self.query_rm_control_sized(cmd, data)
        })
    }
}

fn probe(
    nvml_bounds: PowerLimitBounds,
    nvml_current_mw: u32,
    mut query: impl FnMut(u32, &mut [u8]) -> anyhow::Result<()>,
) -> anyhow::Result<Option<LowerPowerLimit>> {
    if !cfg!(target_endian = "little") {
        return Ok(None);
    }
    // Probe only the two known wire formats, using GETs. A version number is
    // not evidence that the payload still has the same layout or units.
    let mut errors = Vec::new();
    for layout in [EXTENDED_LAYOUT, LEGACY_LAYOUT] {
        let candidate: anyhow::Result<LowerPowerLimit> = (|| {
            let bounds = read_bounds(layout, &mut query)?;
            ensure!(bounds == nvml_bounds, "RM power bounds differ from NVML");
            let control = read_control(layout, &mut query)?;
            ensure!(
                read_u32(&control, layout.request_at) == nvml_current_mw,
                "RM ordinary power request differs from NVML"
            );
            Ok(LowerPowerLimit { bounds, layout })
        })();
        match candidate {
            Ok(support) => return Ok(Some(support)),
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

fn set_limit(
    limit_mw: u32,
    support: LowerPowerLimit,
    mut query: impl FnMut(u32, &mut [u8]) -> anyhow::Result<()>,
) -> anyhow::Result<()> {
    let layout = support.layout;
    let bounds = read_bounds(layout, &mut query)?;
    ensure!(
        bounds == support.bounds,
        "RM power bounds changed since discovery"
    );
    ensure!(
        (bounds.lower_min_mw()..=bounds.max_mw).contains(&limit_mw),
        "Power limit is outside the supported range"
    );

    let before = read_control(layout, &mut query)?;
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
    fn detects_both_layouts_with_gets_without_a_driver_version() {
        for layout in [EXTENDED_LAYOUT, LEGACY_LAYOUT] {
            let mut rm = FakeRm::new(layout, 250_000);
            let support = probe(BOUNDS, 250_000, |c, d| rm.query(c, d))
                .unwrap()
                .unwrap();
            assert_eq!(
                support,
                LowerPowerLimit {
                    bounds: BOUNDS,
                    layout
                }
            );
            let expected = if layout == EXTENDED_LAYOUT {
                vec![(GET_INFO, 0x924), (GET_CONTROL, 0x328)]
            } else {
                vec![(GET_INFO, 0x924), (GET_INFO, 0x488), (GET_CONTROL, 0x188)]
            };
            assert_eq!(rm.reads, expected);
            assert!(rm.writes.is_empty());
        }
    }

    #[test]
    fn unknown_layout_and_nvml_mismatches_never_write() {
        let mut calls = Vec::new();
        assert!(
            probe(BOUNDS, 250_000, |cmd, data| {
                calls.push((cmd, data.len()));
                anyhow::bail!("Unsupported payload")
            })
            .is_err()
        );
        assert_eq!(calls, [(GET_INFO, 0x924), (GET_INFO, 0x488)]);

        for layout in [EXTENDED_LAYOUT, LEGACY_LAYOUT] {
            let mut rm = FakeRm::new(layout, 250_000);
            assert!(probe(BOUNDS, 300_000, |c, d| rm.query(c, d)).is_err());
            let other_bounds = PowerLimitBounds {
                max_mw: 350_000,
                ..BOUNDS
            };
            assert!(probe(other_bounds, 250_000, |c, d| rm.query(c, d)).is_err());
            assert!(rm.writes.is_empty());
        }
    }

    #[test]
    fn rejects_unrecognized_headers_masks_and_client_values() {
        for layout in [EXTENDED_LAYOUT, LEGACY_LAYOUT] {
            for (at, value) in [(0, 0), (4, 3), (layout.client_at, 0xf8)] {
                let mut rm = FakeRm::new(layout, 250_000);
                rm.control[at] = value;
                assert!(probe(BOUNDS, 250_000, |c, d| rm.query(c, d)).is_err());
                assert!(rm.writes.is_empty());
            }
            for current in [0, u32::MAX] {
                let mut rm = FakeRm::new(layout, current);
                assert!(probe(BOUNDS, current, |c, d| rm.query(c, d)).is_err());
            }
        }
        // The larger group has additional mask words. Accepting only its low
        // word would allow an unexpected client to be included in a later SET.
        let mut rm = FakeRm::new(EXTENDED_LAYOUT, 250_000);
        rm.control[8] = 1;
        assert!(probe(BOUNDS, 250_000, |c, d| rm.query(c, d)).is_err());
        assert!(rm.writes.is_empty());
    }

    #[test]
    fn changes_only_fe_request_and_keeps_vbios_maximum() {
        for layout in [EXTENDED_LAYOUT, LEGACY_LAYOUT] {
            let mut rm = FakeRm::new(layout, 250_000);
            let support = probe(BOUNDS, 250_000, |c, d| rm.query(c, d))
                .unwrap()
                .unwrap();
            for cap in [150_000, 30_000, 250_000] {
                set_limit(cap, support, |c, d| rm.query(c, d)).unwrap();
                assert_eq!(read_u32(&rm.control, layout.request_at), cap);
            }
            let writes = rm.writes.len();
            for cap in [0, 29_999, 325_001, 350_000, u32::MAX] {
                assert!(set_limit(cap, support, |c, d| rm.query(c, d)).is_err());
            }
            assert_eq!(rm.writes.len(), writes);
        }
    }

    #[test]
    fn restores_previous_below_minimum_request_after_set_or_readback_failure() {
        for layout in [EXTENDED_LAYOUT, LEGACY_LAYOUT] {
            for fail_set in [false, true] {
                let mut rm = FakeRm::new(layout, 100_000);
                let original = rm.control.clone();
                rm.fail_first_write = fail_set;
                rm.fail_readback = !fail_set;
                let support = LowerPowerLimit {
                    bounds: BOUNDS,
                    layout,
                };
                assert!(set_limit(150_000, support, |c, d| rm.query(c, d)).is_err());
                assert_eq!(rm.writes.len(), 2);
                assert_eq!(rm.control, original);
            }
        }
    }

    #[test]
    fn reports_restore_failure_and_rejects_wrong_client_before_writing() {
        for layout in [EXTENDED_LAYOUT, LEGACY_LAYOUT] {
            let support = LowerPowerLimit {
                bounds: BOUNDS,
                layout,
            };
            let mut rm = FakeRm::new(layout, 100_000);
            rm.fail_first_write = true;
            rm.fail_restore = true;
            let err = set_limit(150_000, support, |c, d| rm.query(c, d)).unwrap_err();
            assert!(err.to_string().contains("restoration also failed"));
            let mut rm = FakeRm::new(layout, 250_000);
            rm.control[layout.client_at] = 0xf8;
            assert!(set_limit(150_000, support, |c, d| rm.query(c, d)).is_err());
            assert!(rm.writes.is_empty());
        }
    }
}
