use super::{NvApi, NvPhysicalGpuHandle, make_version};
use anyhow::{Context, ensure};

const GET_INFO: u32 = 0x2f69f8e5;
const GET_CONTROL: u32 = 0xc4554575;
const SET_CONTROL: u32 = 0xe097144f;
const POLICY_COUNT: usize = 16;
const SLOWDOWN_POLICY: u32 = 4;
const GPU_CHANNEL: u32 = 1;
const TEMPERATURE_SCALE: i32 = 256;

// Undocumented NvAPI client thermal policy ABI. Keep the unknown fields when
// writing; policy flags include the software/hardware slowdown mode.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct PolicyInfo {
    policy_type: u32,
    channel: u32,
    minimum: i32,
    default: i32,
    maximum: i32,
    reserved: [u8; 0x39c - 20],
}

#[repr(C)]
struct PoliciesInfoV4 {
    version: u32,
    mask: u32,
    reserved: [u8; 0x3b4 - 8],
    policies: [PolicyInfo; POLICY_COUNT],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PolicyControl {
    policy_type: u32,
    default_temperature: i32,
    reserved: [u8; 28],
    thresholds: [i32; 4],
    flags: u32,
    reserved_tail: [u8; 4],
}

#[repr(C)]
#[derive(Clone, Debug)]
struct PoliciesControlV3 {
    version: u32,
    mask: u32,
    reserved: [u8; 16],
    policies: [PolicyControl; POLICY_COUNT],
    reserved_tail: [u8; 36],
}

impl Default for PoliciesInfoV4 {
    fn default() -> Self {
        // These ABI structures contain only integers and byte arrays.
        let mut value: Self = unsafe { std::mem::zeroed() };
        value.version = make_version::<Self>(4);
        value
    }
}

impl PoliciesControlV3 {
    fn empty(mask: u32) -> Self {
        let mut value: Self = unsafe { std::mem::zeroed() };
        value.version = make_version::<Self>(3);
        value.mask = mask;
        value
    }
}

pub struct ThermalPolicies {
    initial: PoliciesControlV3,
    gpu_mask: u32,
    software_policy: usize,
    minimum: i32,
}

impl ThermalPolicies {
    pub unsafe fn read(api: &NvApi, handle: NvPhysicalGpuHandle) -> anyhow::Result<Self> {
        let mut info = PoliciesInfoV4::default();
        api.physical_gpu_query(handle, &mut info, GET_INFO)?;
        ensure!(
            info.mask != 0 && info.mask >> POLICY_COUNT == 0,
            "Unsupported thermal policy mask"
        );
        let initial = Self::read_control(api, handle, info.mask)?;
        Self::from_info(&info, initial)
    }

    fn from_info(info: &PoliciesInfoV4, initial: PoliciesControlV3) -> anyhow::Result<Self> {
        let mut gpu_mask = 0;
        let mut software_policy = None;
        for (index, policy) in info.policies.iter().enumerate() {
            if info.mask & (1 << index) == 0
                || policy.policy_type != SLOWDOWN_POLICY
                || policy.channel != GPU_CHANNEL
            {
                continue;
            }
            let control = initial.policies[index];
            ensure!(
                control.policy_type == SLOWDOWN_POLICY,
                "Thermal policy type mismatch"
            );
            ensure!(
                control.thresholds.windows(2).all(|pair| pair[0] >= pair[1]),
                "Unexpected thermal hysteresis"
            );
            gpu_mask |= 1 << index;
            if control.flags & 1 != 0 {
                ensure!(
                    software_policy.is_none(),
                    "Multiple GPU software slowdown policies"
                );
                software_policy = Some(index);
            }
        }
        let software_policy = software_policy.context("No GPU software slowdown policy")?;
        Ok(Self {
            initial,
            gpu_mask,
            software_policy,
            minimum: info.policies[software_policy].minimum,
        })
    }

    unsafe fn read_control(
        api: &NvApi,
        handle: NvPhysicalGpuHandle,
        mask: u32,
    ) -> anyhow::Result<PoliciesControlV3> {
        let mut value = PoliciesControlV3::empty(mask);
        api.physical_gpu_query(handle, &mut value, GET_CONTROL)?;
        ensure!(value.mask == mask, "Thermal policy mask changed");
        Ok(value)
    }

    fn control_for_temperature(
        &self,
        current: &PoliciesControlV3,
        target: u32,
        hardware_limit: u32,
    ) -> anyhow::Result<PoliciesControlV3> {
        let target = i32::try_from(target)
            .ok()
            .and_then(|value| value.checked_mul(TEMPERATURE_SCALE))
            .context("Thermal temperature exceeds the fixed-point range")?;
        let limit = i32::try_from(hardware_limit)
            .ok()
            .and_then(|value| value.checked_mul(TEMPERATURE_SCALE))
            .context("Hardware thermal limit exceeds the fixed-point range")?;
        ensure!(
            target >= self.minimum,
            "Temperature is below the reported policy minimum"
        );
        let delta = target
            .checked_sub(self.initial.policies[self.software_policy].thresholds[1])
            .context("Thermal temperature delta overflow")?;
        let mut requested = current.clone();
        requested.mask = self.gpu_mask;
        for index in 0..POLICY_COUNT {
            if self.gpu_mask & (1 << index) == 0 {
                continue;
            }
            for (threshold, initial) in requested.policies[index]
                .thresholds
                .iter_mut()
                .zip(self.initial.policies[index].thresholds)
            {
                *threshold = initial
                    .checked_add(delta)
                    .context("Thermal threshold overflow")?;
                ensure!(
                    *threshold >= 0 && *threshold < limit,
                    "Shifted threshold reaches the hardware thermal limit"
                );
            }
        }
        Ok(requested)
    }

    unsafe fn write_control(
        &self,
        api: &NvApi,
        handle: NvPhysicalGpuHandle,
        mut requested: PoliciesControlV3,
    ) -> anyhow::Result<()> {
        api.physical_gpu_query(handle, &mut requested, SET_CONTROL)?;
        let actual = Self::read_control(api, handle, self.initial.mask)?;
        for index in 0..POLICY_COUNT {
            if self.initial.mask & (1 << index) == 0 {
                continue;
            }
            let want = requested.policies[index];
            let got = actual.policies[index];
            ensure!(
                got.policy_type == want.policy_type
                    && got.flags == want.flags
                    && got.thresholds == want.thresholds,
                "Thermal policy {index} readback differs from the request"
            );
        }
        Ok(())
    }

    pub unsafe fn apply(
        &self,
        api: &NvApi,
        handle: NvPhysicalGpuHandle,
        target: u32,
        hardware_limit: u32,
    ) -> anyhow::Result<()> {
        let current = Self::read_control(api, handle, self.initial.mask)?;
        let requested = self.control_for_temperature(&current, target, hardware_limit)?;
        self.write_control(api, handle, requested)
    }

    pub unsafe fn restore(&self, api: &NvApi, handle: NvPhysicalGpuHandle) -> anyhow::Result<()> {
        let mut current = Self::read_control(api, handle, self.initial.mask)?;
        current.mask = self.gpu_mask;
        for index in 0..POLICY_COUNT {
            if self.gpu_mask & (1 << index) != 0 {
                current.policies[index].thresholds = self.initial.policies[index].thresholds;
            }
        }
        self.write_control(api, handle, current)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{offset_of, size_of};

    fn fixture() -> ThermalPolicies {
        let mut info = PoliciesInfoV4::default();
        info.mask = 0b10111;
        let mut control = PoliciesControlV3::empty(info.mask);
        for (index, channel, mode, center) in
            [(0, 1, 0, 89), (1, 0, 0, 105), (2, 1, 1, 87), (4, 2, 0, 103)]
        {
            info.policies[index].policy_type = SLOWDOWN_POLICY;
            info.policies[index].channel = channel;
            info.policies[index].minimum = 75 * 256;
            control.policies[index].policy_type = SLOWDOWN_POLICY;
            control.policies[index].flags = 0xa0 | mode;
            control.policies[index].reserved = [0x5a; 28];
            control.policies[index].thresholds =
                [center + 1, center, center - 1, center - 3].map(|t| t * 256);
        }
        ThermalPolicies::from_info(&info, control).unwrap()
    }

    #[test]
    fn thermal_policy_abi_layout() {
        assert_eq!(size_of::<PoliciesInfoV4>(), 0x3d74);
        assert_eq!(offset_of!(PoliciesInfoV4, policies), 0x3b4);
        assert_eq!(size_of::<PolicyInfo>(), 0x39c);
        assert_eq!(size_of::<PoliciesControlV3>(), 0x3fc);
        assert_eq!(offset_of!(PoliciesControlV3, policies), 0x18);
        assert_eq!(size_of::<PolicyControl>(), 0x3c);
        assert_eq!(offset_of!(PolicyControl, thresholds), 0x24);
        assert_eq!(offset_of!(PolicyControl, flags), 0x34);
    }

    #[test]
    fn shifts_gpu_policies_together_and_preserves_other_fields() {
        let state = fixture();
        let mut expected = state.initial.clone();
        expected.mask = 0b101;
        expected.policies[0].thresholds = [96, 95, 94, 92].map(|t| t * 256);
        expected.policies[2].thresholds = [94, 93, 92, 90].map(|t| t * 256);
        let actual = state
            .control_for_temperature(&state.initial, 93, 99)
            .unwrap();
        assert_eq!(actual.mask, expected.mask);
        assert_eq!(actual.policies, expected.policies);
    }

    #[test]
    fn uses_initial_thresholds_and_preserves_current_unrelated_fields() {
        let state = fixture();
        let mut current = state.initial.clone();
        current.policies[2].thresholds = [94, 93, 92, 90].map(|t| t * 256);
        current.policies[4].thresholds = [107, 106, 105, 103].map(|t| t * 256);
        let actual = state.control_for_temperature(&current, 91, 99).unwrap();
        assert_eq!(
            actual.policies[2].thresholds,
            [92, 91, 90, 88].map(|t| t * 256)
        );
        assert_eq!(actual.policies[4], current.policies[4]);
    }

    #[test]
    fn rejects_overflow_and_thresholds_past_hardware_limits() {
        let state = fixture();
        for target in [0, 74, 96, u32::MAX] {
            assert!(
                state
                    .control_for_temperature(&state.initial, target, 99)
                    .is_err()
            );
        }
    }

    #[test]
    #[ignore = "Requires an NVIDIA GPU supporting the private thermal policy ABI; read-only"]
    fn live_thermal_policy_read() {
        let api = NvApi::new().unwrap();
        let bus = std::env::var("LACT_TEST_NVIDIA_BUS")
            .unwrap()
            .parse()
            .unwrap();
        let handle = api.find_matching_gpu(bus).unwrap().unwrap();
        let state = unsafe { ThermalPolicies::read(&api, handle) }.unwrap();
        let target = state.initial.policies[state.software_policy].thresholds[1] / 256;
        println!(
            "GPU slowdown mask={:#x}, software policy={}, current={}C",
            state.gpu_mask, state.software_policy, target
        );
    }

    #[test]
    #[ignore = "Explicit hardware test: lowers the policy temperature by 1C, then restores it"]
    fn live_thermal_policy_round_trip() {
        use nvml_wrapper::{Nvml, enum_wrappers::device::TemperatureThreshold};

        let api = NvApi::new().unwrap();
        let bus: u32 = std::env::var("LACT_TEST_NVIDIA_BUS")
            .unwrap()
            .parse()
            .unwrap();
        let handle = api.find_matching_gpu(bus).unwrap().unwrap();
        let nvml = Nvml::init().unwrap();
        let pci = format!("0000:{bus:02x}:00.0");
        let limit = nvml
            .device_by_pci_bus_id(pci.as_str())
            .unwrap()
            .temperature_threshold(TemperatureThreshold::Slowdown)
            .unwrap();
        let state = unsafe { ThermalPolicies::read(&api, handle) }.unwrap();
        let initial = state.initial.policies[state.software_policy].thresholds[1];
        let target = u32::try_from(initial / 256 - 1).unwrap();
        let applied = (|| unsafe {
            state.apply(&api, handle, target, limit)?;
            let actual = ThermalPolicies::read_control(&api, handle, state.initial.mask)?;
            ensure!(
                actual.policies[state.software_policy].thresholds[1]
                    == i32::try_from(target).unwrap() * 256,
                "New slowdown temperature was not retained"
            );
            Ok::<_, anyhow::Error>(())
        })();
        // Always attempt restoration, including after an unsuccessful setter.
        unsafe { state.restore(&api, handle) }.unwrap();
        applied.unwrap();
        let restored =
            unsafe { ThermalPolicies::read_control(&api, handle, state.initial.mask) }.unwrap();
        assert_eq!(restored.policies, state.initial.policies);
        println!(
            "Verified {}C -> {target}C -> {}C",
            initial / 256,
            initial / 256
        );
    }
}
