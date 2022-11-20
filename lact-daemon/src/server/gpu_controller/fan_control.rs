use amdgpu_sysfs::hw_mon::Temperature;
use lact_schema::FanCurveMap;
use serde::{Deserialize, Serialize};
use tracing::error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FanCurve(pub FanCurveMap);

impl FanCurve {
    pub fn rpm_at_temp(&self, temp: Temperature, min_rpm: u32, max_rpm: u32) -> u32 {
        let current = temp.current.expect("No current temp");

        // This scenario is most likely unreachable as the kernel shuts down the GPU when it reaches critical temperature
        if temp.crit.filter(|crit| current > *crit).is_some()
            || temp.crit_hyst.filter(|hyst| current < *hyst).is_some()
        {
            error!("GPU temperature is beyond critical values! {current}°C");
            return max_rpm;
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

        ((max_rpm - min_rpm) as f32 * percentage) as u32
    }
}

impl Default for FanCurve {
    fn default() -> Self {
        Self(
            [
                (30, 0.0),
                (40, 0.2),
                (50, 0.35),
                (60, 0.5),
                (70, 0.75),
                (80, 1.0),
            ]
            .into(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::FanCurve;
    use amdgpu_sysfs::hw_mon::Temperature;

    fn simple_rpm(temp: f32, min_rpm: u32, max_rpm: u32) -> u32 {
        let curve = FanCurve([(0, 0.0), (100, 1.0)].into());
        let temp = Temperature {
            current: Some(temp),
            crit: Some(150.0),
            crit_hyst: Some(-100.0),
        };
        curve.rpm_at_temp(temp, min_rpm, max_rpm)
    }

    #[test]
    fn simple_curve_middle() {
        let rpm = simple_rpm(45.0, 0, 200);
        assert_eq!(rpm, 90);
    }

    #[test]
    fn simple_curve_start() {
        let rpm = simple_rpm(0.0, 0, 200);
        assert_eq!(rpm, 0);
    }

    #[test]
    fn simple_curve_end() {
        let rpm = simple_rpm(100.0, 0, 200);
        assert_eq!(rpm, 200);
    }

    #[test]
    fn simple_curve_before() {
        let rpm = simple_rpm(-5.0, 0, 200);
        assert_eq!(rpm, 0);
    }

    #[test]
    fn simple_curve_after() {
        let rpm = simple_rpm(105.0, 0, 200);
        assert_eq!(rpm, 200);
    }

    #[test]
    fn curve_crit() {
        let curve = FanCurve([(20, 0.0), (80, 100.0)].into());
        let temp = Temperature {
            current: Some(100.0),
            crit: Some(90.0),
            crit_hyst: Some(0.0),
        };
        let rpm = curve.rpm_at_temp(temp, 0, 200);
        assert_eq!(rpm, 200);
    }

    #[test]
    fn default_curve() {
        let curve = FanCurve::default();
        let rpm_at_temp = |current: f32| {
            let temp = Temperature {
                current: Some(current),
                crit: Some(90.0),
                crit_hyst: Some(0.0),
            };
            curve.rpm_at_temp(temp, 0, 1000)
        };
        assert_eq!(rpm_at_temp(20.0), 0);
        assert_eq!(rpm_at_temp(30.0), 0);
        assert_eq!(rpm_at_temp(33.0), 60);
        assert_eq!(rpm_at_temp(60.0), 500);
        assert_eq!(rpm_at_temp(65.0), 625);
        assert_eq!(rpm_at_temp(70.0), 750);
        assert_eq!(rpm_at_temp(79.0), 975);
        assert_eq!(rpm_at_temp(85.0), 1000);
        assert_eq!(rpm_at_temp(100.0), 1000);
        assert_eq!(rpm_at_temp(-5.0), 1000);
    }
}
