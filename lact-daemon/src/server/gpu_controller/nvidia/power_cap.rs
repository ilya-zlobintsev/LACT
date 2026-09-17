use super::driver::power_limit::PowerLimitBounds;
use anyhow::{Context, ensure};
use lact_schema::config::NvidiaPowerCapMode;

/// Keep backend selection testable without invoking either hardware interface.
pub(super) trait PowerCapControl {
    type IoctlSupport;

    fn range(&self) -> anyhow::Result<(u32, u32)>;
    fn current(&self) -> anyhow::Result<u32>;
    fn default_cap(&self) -> anyhow::Result<u32>;
    fn set_nvml(&mut self, cap: u32) -> anyhow::Result<()>;
    fn probe_ioctl(
        &self,
        bounds: PowerLimitBounds,
        current: u32,
    ) -> anyhow::Result<Self::IoctlSupport>;
    fn set_ioctl(&self, cap: u32, support: &Self::IoctlSupport) -> anyhow::Result<()>;
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn checked_cap_mw(cap: f64, min: u32, max: u32) -> anyhow::Result<u32> {
    ensure!(
        cap.is_finite()
            && cap > 0.0
            && cap >= f64::from(min) / 1000.0
            && cap <= f64::from(max) / 1000.0,
        "Power cap must be between {} and {} W",
        f64::from(min) / 1000.0,
        f64::from(max) / 1000.0,
    );
    Ok((cap * 1000.0) as u32)
}

pub(super) fn apply<C: PowerCapControl>(
    control: &mut C,
    cap: Option<f64>,
    mode: NvidiaPowerCapMode,
) -> anyhow::Result<Option<C::IoctlSupport>> {
    match mode {
        NvidiaPowerCapMode::Nvml => {
            if let Some(cap) = cap {
                let (min, max) = control
                    .range()
                    .context("Could not get power cap constraints")?;
                let cap = checked_cap_mw(cap, min, max)?;
                if control.current().context("Could not get current cap")? != cap {
                    control
                        .set_nvml(cap)
                        .context("Could not set power cap using NVML")?;
                }
            } else if let (Ok(current), Ok(default)) = (control.current(), control.default_cap())
                && current != default
            {
                control
                    .set_nvml(default)
                    .context("Could not reset power cap using NVML")?;
            }
            Ok(None)
        }
        NvidiaPowerCapMode::Ioctl => {
            let (min_mw, max_mw) = control
                .range()
                .context("Could not get power cap constraints")?;
            let bounds = PowerLimitBounds {
                min_mw,
                max_mw,
                default_mw: control
                    .default_cap()
                    .context("Could not get default power cap")?,
            };
            let cap = cap.unwrap_or(f64::from(bounds.default_mw) / 1000.0);
            let cap = checked_cap_mw(cap, bounds.lower_min_mw(), bounds.max_mw)?;
            let current = control.current().context("Could not get current cap")?;
            let support = control
                .probe_ioctl(bounds, current)
                .context("Experimental NVIDIA power control is not supported")?;
            // This route handles every value, including native-range caps and reset.
            // Failure is reported; never silently switch back to NVML.
            control
                .set_ioctl(cap, &support)
                .context("Could not set power cap using ioctl")?;
            Ok(Some(support))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct FakeControl {
        current: u32,
        calls: RefCell<Vec<(&'static str, u32)>>,
        reject_probe: bool,
        reject_nvml: bool,
        reject_ioctl: bool,
        no_power_control: bool,
    }

    impl Default for FakeControl {
        fn default() -> Self {
            Self {
                current: 275_000,
                calls: RefCell::default(),
                reject_probe: false,
                reject_nvml: false,
                reject_ioctl: false,
                no_power_control: false,
            }
        }
    }

    impl PowerCapControl for FakeControl {
        type IoctlSupport = ();
        fn range(&self) -> anyhow::Result<(u32, u32)> {
            Ok((250_000, 325_000))
        }
        fn current(&self) -> anyhow::Result<u32> {
            ensure!(!self.no_power_control, "unsupported");
            Ok(self.current)
        }
        fn default_cap(&self) -> anyhow::Result<u32> {
            ensure!(!self.no_power_control, "unsupported");
            Ok(300_000)
        }
        fn set_nvml(&mut self, cap: u32) -> anyhow::Result<()> {
            self.calls.borrow_mut().push(("nvml", cap));
            ensure!(!self.reject_nvml, "NVML failure");
            Ok(())
        }
        fn probe_ioctl(&self, bounds: PowerLimitBounds, current: u32) -> anyhow::Result<()> {
            assert_eq!(
                bounds,
                PowerLimitBounds {
                    min_mw: 250_000,
                    default_mw: 300_000,
                    max_mw: 325_000
                }
            );
            assert_eq!(current, self.current);
            self.calls.borrow_mut().push(("probe", current));
            ensure!(!self.reject_probe, "unknown layout");
            Ok(())
        }
        fn set_ioctl(&self, cap: u32, _: &()) -> anyhow::Result<()> {
            self.calls.borrow_mut().push(("ioctl", cap));
            ensure!(!self.reject_ioctl, "ioctl failure");
            Ok(())
        }
    }

    #[test]
    fn default_mode_never_probes_or_writes_ioctl() {
        for cap in [Some(250.0), Some(300.0), Some(325.0), None] {
            let mut control = FakeControl::default();
            assert!(
                apply(&mut control, cap, NvidiaPowerCapMode::Nvml)
                    .unwrap()
                    .is_none()
            );
            let expected = cap.map_or(300_000, |cap| {
                checked_cap_mw(cap, 250_000, 325_000).unwrap()
            });
            assert_eq!(*control.calls.borrow(), [("nvml", expected)]);
        }
        let mut control = FakeControl::default();
        assert!(apply(&mut control, Some(150.0), NvidiaPowerCapMode::Nvml).is_err());
        assert!(control.calls.borrow().is_empty());
    }

    #[test]
    fn opt_in_uses_ioctl_for_lower_native_and_default_caps() {
        for cap in [Some(30.0), Some(150.0), Some(275.0), Some(325.0), None] {
            let mut control = FakeControl::default();
            assert!(
                apply(&mut control, cap, NvidiaPowerCapMode::Ioctl)
                    .unwrap()
                    .is_some()
            );
            let expected = cap.map_or(300_000, |cap| checked_cap_mw(cap, 30_000, 325_000).unwrap());
            assert_eq!(
                *control.calls.borrow(),
                [("probe", 275_000), ("ioctl", expected)]
            );
        }
    }

    #[test]
    fn backend_errors_do_not_change_the_selected_mode() {
        let mut nvml = FakeControl {
            reject_nvml: true,
            ..Default::default()
        };
        assert!(apply(&mut nvml, Some(300.0), NvidiaPowerCapMode::Nvml).is_err());
        assert_eq!(*nvml.calls.borrow(), [("nvml", 300_000)]);

        let mut unknown = FakeControl {
            reject_probe: true,
            ..Default::default()
        };
        assert!(apply(&mut unknown, Some(300.0), NvidiaPowerCapMode::Ioctl).is_err());
        assert_eq!(*unknown.calls.borrow(), [("probe", 275_000)]);

        let mut ioctl = FakeControl {
            reject_ioctl: true,
            ..Default::default()
        };
        assert!(apply(&mut ioctl, None, NvidiaPowerCapMode::Ioctl).is_err());
        assert_eq!(
            *ioctl.calls.borrow(),
            [("probe", 275_000), ("ioctl", 300_000)]
        );
    }

    #[test]
    fn opting_out_can_restore_a_previous_below_minimum_cap_with_nvml() {
        let mut control = FakeControl {
            current: 150_000,
            ..Default::default()
        };
        apply(&mut control, None, NvidiaPowerCapMode::Nvml).unwrap();
        assert_eq!(*control.calls.borrow(), [("nvml", 300_000)]);
    }

    #[test]
    fn default_mode_keeps_reset_optional_on_devices_without_power_control() {
        let mut control = FakeControl {
            no_power_control: true,
            ..Default::default()
        };
        apply(&mut control, None, NvidiaPowerCapMode::Nvml).unwrap();
        assert!(control.calls.borrow().is_empty());
    }

    #[test]
    fn power_cap_validates_before_converting_to_milliwatts() {
        assert_eq!(checked_cap_mw(150.125, 30_000, 325_000).unwrap(), 150_125);
        for cap in [
            f64::NAN,
            f64::INFINITY,
            f64::NEG_INFINITY,
            -1.0,
            0.0,
            29.999,
            325.001,
            8000.0,
        ] {
            for mode in [NvidiaPowerCapMode::Nvml, NvidiaPowerCapMode::Ioctl] {
                let mut control = FakeControl::default();
                assert!(apply(&mut control, Some(cap), mode).is_err());
                assert!(control.calls.borrow().is_empty());
            }
        }
    }
}
