use amdgpu_sysfs::hw_mon::Temperature;
use anyhow::anyhow;
use lact_schema::{default_fan_curve, FanCurveMap};
use serde::{Deserialize, Serialize};
use tracing::warn;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FanCurve(pub FanCurveMap);

impl FanCurve {
    pub fn pwm_at_temp(&self, temp: Temperature) -> u8 {
        let current = temp.current.expect("No current temp");

        // This scenario is most likely unreachable as the kernel shuts down the GPU when it reaches critical temperature
        if temp.crit.filter(|crit| current > *crit).is_some()
            || temp.crit_hyst.filter(|hyst| current < *hyst).is_some()
        {
            warn!("GPU temperature is beyond critical values! {current}°C");
            return u8::MAX;
        }

        let current = current as i32;
        let maybe_lower = self.0.range(..current).next_back();
        let maybe_higher = self.0.range(current..).next();

        let percentage = match (maybe_lower, maybe_higher) {
            (Some((lower_temp, lower_speed)), Some((higher_temp, higher_speed))) => {
                let speed_ratio = (current - lower_temp) as f32 / (higher_temp - lower_temp) as f32;
                lower_speed + (higher_speed - lower_speed) * speed_ratio
            }
            (Some((_, lower_speed)), None) => *lower_speed,
            (None, Some((_, higher_speed))) => *higher_speed,
            (None, None) => panic!("Could not find fan speed on the curve! This is a bug."),
        };

        (u8::MAX as f32 * percentage) as u8
    }
}

impl FanCurve {
    pub fn validate(&self) -> anyhow::Result<()> {
        for percentage in self.0.values() {
            if !(0.0..=1.0).contains(percentage) {
                return Err(anyhow!("Fan speed percentage must be between 0 and 1"));
            }
        }
        Ok(())
    }
}

impl Default for FanCurve {
    fn default() -> Self {
        Self(default_fan_curve())
    }
}

#[cfg(test)]
mod tests {
    use super::FanCurve;
    use amdgpu_sysfs::hw_mon::Temperature;

    fn simple_pwm(temp: f32) -> u8 {
        let curve = FanCurve([(0, 0.0), (100, 1.0)].into());
        let temp = Temperature {
            current: Some(temp),
            crit: Some(150.0),
            crit_hyst: Some(-100.0),
        };
        curve.pwm_at_temp(temp)
    }

    #[test]
    fn simple_curve_middle() {
        let pwm = simple_pwm(45.0);
        assert_eq!(pwm, 114);
    }

    #[test]
    fn simple_curve_start() {
        let pwm = simple_pwm(0.0);
        assert_eq!(pwm, 0);
    }

    #[test]
    fn simple_curve_end() {
        let pwm = simple_pwm(100.0);
        assert_eq!(pwm, 255);
    }

    #[test]
    fn simple_curve_before() {
        let pwm = simple_pwm(-5.0);
        assert_eq!(pwm, 0);
    }

    #[test]
    fn simple_curve_after() {
        let pwm = simple_pwm(105.0);
        assert_eq!(pwm, 255);
    }

    #[test]
    fn curve_crit() {
        let curve = FanCurve([(20, 0.0), (80, 100.0)].into());
        let temp = Temperature {
            current: Some(100.0),
            crit: Some(90.0),
            crit_hyst: Some(0.0),
        };
        let pwm = curve.pwm_at_temp(temp);
        assert_eq!(pwm, 255);
    }

    #[test]
    fn uneven_curve() {
        let curve = FanCurve([(30, 0.0), (40, 0.1), (55, 0.9), (61, 1.0)].into());
        let pwm_at_temp = |current: f32| {
            let temp = Temperature {
                current: Some(current),
                crit: Some(90.0),
                crit_hyst: Some(0.0),
            };
            curve.pwm_at_temp(temp)
        };

        assert_eq!(pwm_at_temp(30.0), 0);
        assert_eq!(pwm_at_temp(35.0), 12);
        assert_eq!(pwm_at_temp(40.0), 25);
        assert_eq!(pwm_at_temp(47.0), 120);
        assert_eq!(pwm_at_temp(52.0), 188);
        assert_eq!(pwm_at_temp(53.0), 202);
        assert_eq!(pwm_at_temp(54.0), 215);
    }

    #[test]
    fn default_curve() {
        let curve = FanCurve::default();
        let pwm_at_temp = |current: f32| {
            let temp = Temperature {
                current: Some(current),
                crit: Some(90.0),
                crit_hyst: Some(0.0),
            };
            curve.pwm_at_temp(temp)
        };
        assert_eq!(pwm_at_temp(20.0), 0);
        assert_eq!(pwm_at_temp(30.0), 0);
        assert_eq!(pwm_at_temp(33.0), 15);
        assert_eq!(pwm_at_temp(60.0), 127);
        assert_eq!(pwm_at_temp(65.0), 159);
        assert_eq!(pwm_at_temp(70.0), 191);
        assert_eq!(pwm_at_temp(79.0), 248);
        assert_eq!(pwm_at_temp(85.0), 255);
        assert_eq!(pwm_at_temp(100.0), 255);
        assert_eq!(pwm_at_temp(-5.0), 255);
    }
}
