use bitflags::bitflags;
use nvml_wrapper::bitmasks::device::ThrottleReasons;
use std::collections::BTreeMap;

bitflags! {
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub struct PerfPolicies: u32 {
        const POWER = 1 << 0;
        const THERMAL = 1 << 1;
        const VREL = 1 << 2;
        const VOP = 1 << 3;
        const UTIL = 1 << 4;
        const SLI = 1 << 5;
    }
}

pub fn from_policies(policies: PerfPolicies) -> BTreeMap<String, Vec<String>> {
    [
        (PerfPolicies::POWER, "Power"),
        (PerfPolicies::THERMAL, "Thermal"),
        (PerfPolicies::VREL, "vRel"),
        (PerfPolicies::VOP, "vOp"),
        (PerfPolicies::SLI, "SLI"),
    ]
    .into_iter()
    .filter(|(policy, _)| policies.contains(*policy))
    .map(|(_, name)| (name.to_owned(), vec![]))
    .collect()
}

pub fn from_reasons(reasons: ThrottleReasons) -> BTreeMap<String, Vec<String>> {
    let mut info: BTreeMap<String, Vec<String>> = [
        (ThrottleReasons::SW_POWER_CAP, "Power"),
        (ThrottleReasons::SW_THERMAL_SLOWDOWN, "Thermal"),
        (ThrottleReasons::SYNC_BOOST, "Sync Boost"),
        (ThrottleReasons::APPLICATIONS_CLOCKS_SETTING, "App Clocks"),
        (ThrottleReasons::DISPLAY_CLOCK_SETTING, "Display Clocks"),
    ]
    .into_iter()
    .filter(|(reason, _)| reasons.contains(*reason))
    .map(|(_, name)| (name.to_owned(), vec![]))
    .collect();

    let slowdown_details: Vec<String> = [
        (ThrottleReasons::HW_THERMAL_SLOWDOWN, "Thermal"),
        (ThrottleReasons::HW_POWER_BRAKE_SLOWDOWN, "Power brake"),
    ]
    .into_iter()
    .filter(|(reason, _)| reasons.contains(*reason))
    .map(|(_, detail)| detail.to_owned())
    .collect();

    if reasons.contains(ThrottleReasons::HW_SLOWDOWN) || !slowdown_details.is_empty() {
        info.insert("HW Slowdown".to_owned(), slowdown_details);
    }

    info
}
