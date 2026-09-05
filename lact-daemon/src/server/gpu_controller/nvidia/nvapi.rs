#![cfg_attr(test, allow(unused))]
#![allow(
    clippy::unreadable_literal,
    unsafe_op_in_unsafe_fn,
    clippy::large_stack_arrays
)]

use crate::bindings::nvidia::{
    NVAPI_MAX_PHYSICAL_GPUS, NVAPI_SHORT_STRING_MAX, NvAPI_Status, NvPhysicalGpuHandle, NvS32,
    NvU8, NvU16, NvU32, NvU64,
};
use anyhow::{Context, bail};
use nvml_wrapper::enums::device::DeviceArchitecture;
use std::{
    ffi::{CStr, c_char, c_uint},
    mem::{self, transmute},
    ptr,
};
use tracing::error;

const LIBARY_NAME: &str = "libnvidia-api.so.1";
const QUERY_INTERFACE_FN: &[u8] = b"nvapi_QueryInterface\0";

const QUERY_NVAPI_INITIALIZE: u32 = 0x0150e828;
const QUERY_NVAPI_UNLOAD: u32 = 0xd22bdd7e;
const QUERY_NVAPI_ENUM_PHYSICAL_GPUS: u32 = 0xe5ac921f;
const QUERY_NVAPI_GPU_GET_BUS_ID: u32 = 0x1be0b8e5;
const QUERY_NVAPI_GET_ERROR_MESSAGE: u32 = 0x6c2d048c;
// Undocumented calls
const QUERY_NVAPI_GPU_THERM_CHANNEL_GET_STATUS: u32 = 0x65fe3aad;
const QUERY_NVAPI_GPU_CLIENT_VOLT_RAILS_GET_STATUS: u32 = 0x465f9bcf;
const QUERY_NVAPI_GPU_CLIENT_VOLT_RAILS_GET_CONTROL: u32 = 0x9df23ca1;
const QUERY_NVAPI_GPU_CLIENT_VOLT_RAILS_SET_CONTROL: u32 = 0xb9306d9b;
const QUERY_NVAPI_GPU_CLOCK_CLIENT_CLK_VF_POINTS_GET_STATUS: u32 = 0x21537ad4;
const QUERY_NVAPI_GPU_CLOCK_CLIENT_CLK_VF_POINTS_GET_INFO: u32 = 0x507b4b59;
const QUERY_NVAPI_GPU_CLOCK_CLIENT_CLK_VF_POINTS_SET_CONTROL: u32 = 0x733e009;
const QUERY_NVAPI_GPU_CLOCK_CLIENT_CLK_VF_POINTS_GET_CONTROL: u32 = 0x23f1b133;
const QUERY_NVAPI_GPU_REGISTER_OP: u32 = 0x2eb3c140;
const QUERY_NVAPI_GPU_GET_ALL_CLOCKS: u32 = 0x1bd69f49;

const REG_OFFSET_BLACKWELL_HOTSPOT_AGGREGATED: u32 = 0xad0aa0;

pub const CLOCK_CLIENT_CLK_VF_POINT_TYPE_PROG: NvU32 = 0;

pub struct NvApi {
    lib: libloading::Library,
}

impl NvApi {
    pub fn new() -> anyhow::Result<Self> {
        let lib = unsafe {
            libloading::Library::new(LIBARY_NAME).context("Could not load nvidia API library")
        }?;

        let handle = Self { lib };

        unsafe {
            let initialize = handle.query_interface(QUERY_NVAPI_INITIALIZE)?;
            let initialize: unsafe extern "C" fn() -> NvAPI_Status = transmute(initialize);
            let status = initialize();
            handle.handle_status(status)?;

            handle.enum_physical_gpus().unwrap();
        }

        Ok(handle)
    }

    pub fn find_matching_gpu(&self, bus_id: u32) -> anyhow::Result<Option<NvPhysicalGpuHandle>> {
        unsafe {
            let handles = self.enum_physical_gpus()?;
            for handle in handles {
                let f = self.query_interface(QUERY_NVAPI_GPU_GET_BUS_ID)?;
                let f: unsafe extern "C" fn(
                    handle: NvPhysicalGpuHandle,
                    id: &mut u32,
                ) -> NvAPI_Status = transmute(f);

                let mut id = 0;
                let status = f(handle, &mut id);
                self.handle_status(status)?;

                if id == bus_id {
                    return Ok(Some(handle));
                }
            }
        }

        Ok(None)
    }

    pub unsafe fn therm_channel_get_status(
        &self,
        handle: NvPhysicalGpuHandle,
        mask: i32,
    ) -> anyhow::Result<NvApiThermals> {
        let f = self.query_interface(QUERY_NVAPI_GPU_THERM_CHANNEL_GET_STATUS)?;
        let f: unsafe extern "C" fn(
            handle: NvPhysicalGpuHandle,
            sensors: &mut NvApiThermals,
        ) -> NvAPI_Status = transmute(f);

        let mut sensors = NvApiThermals {
            #[allow(clippy::cast_possible_truncation)]
            version: make_version::<NvApiThermals>(2),
            mask,
            values: [0; 40],
        };

        let status = f(handle, &mut sensors);
        self.handle_status(status)?;

        Ok(sensors)
    }

    pub unsafe fn client_volt_rails_get_status(
        &self,
        handle: NvPhysicalGpuHandle,
    ) -> anyhow::Result<u32> {
        let mut data = ClientVoltRailsStatusV1::default();
        self.physical_gpu_query(
            handle,
            &mut data,
            QUERY_NVAPI_GPU_CLIENT_VOLT_RAILS_GET_STATUS,
        )?;

        Ok(data.rails[0].current_voltage_uv)
    }

    pub unsafe fn client_volt_rails_get_control(
        &self,
        handle: NvPhysicalGpuHandle,
    ) -> anyhow::Result<u8> {
        let mut data = ClientVoltRailsControlV1::default();
        self.physical_gpu_query(
            handle,
            &mut data,
            QUERY_NVAPI_GPU_CLIENT_VOLT_RAILS_GET_CONTROL,
        )?;

        Ok(data.percent_delta)
    }

    pub unsafe fn get_all_clocks(
        &self,
        handle: NvPhysicalGpuHandle,
    ) -> anyhow::Result<NvGpuClockInfoV2> {
        let mut data = NvGpuClockInfoV2::default();
        self.physical_gpu_query(handle, &mut data, QUERY_NVAPI_GPU_GET_ALL_CLOCKS)?;

        Ok(data)
    }

    pub unsafe fn client_volt_rails_set_control(
        &self,
        handle: NvPhysicalGpuHandle,
        percent: u8,
    ) -> anyhow::Result<()> {
        let mut data = ClientVoltRailsControlV1 {
            percent_delta: percent,
            ..Default::default()
        };
        self.physical_gpu_query(
            handle,
            &mut data,
            QUERY_NVAPI_GPU_CLIENT_VOLT_RAILS_SET_CONTROL,
        )?;

        Ok(())
    }

    pub unsafe fn clock_client_clk_vf_points_get_info(
        &self,
        handle: NvPhysicalGpuHandle,
    ) -> anyhow::Result<ClockClientClkVfPointsInfoV1> {
        let mut data = ClockClientClkVfPointsInfoV1::default();

        self.physical_gpu_query(
            handle,
            &mut data,
            QUERY_NVAPI_GPU_CLOCK_CLIENT_CLK_VF_POINTS_GET_INFO,
        )?;

        Ok(data)
    }

    pub unsafe fn clock_client_clk_vf_points_get_status(
        &self,
        handle: NvPhysicalGpuHandle,
        vf_points_mask: [NvU32; 8],
    ) -> anyhow::Result<ClockClientClkVfPointsStatusV3> {
        let mut data = ClockClientClkVfPointsStatusV3 {
            vf_points_mask,
            ..Default::default()
        };

        self.physical_gpu_query(
            handle,
            &mut data,
            QUERY_NVAPI_GPU_CLOCK_CLIENT_CLK_VF_POINTS_GET_STATUS,
        )?;

        Ok(data)
    }

    pub unsafe fn clock_client_clk_vf_points_get_control(
        &self,
        handle: NvPhysicalGpuHandle,
        vf_points_mask: [NvU32; 8],
    ) -> anyhow::Result<ClockClientClkVfPointsControlV1> {
        let mut data = ClockClientClkVfPointsControlV1 {
            vf_points_mask,
            ..Default::default()
        };

        self.physical_gpu_query(
            handle,
            &mut data,
            QUERY_NVAPI_GPU_CLOCK_CLIENT_CLK_VF_POINTS_GET_CONTROL,
        )?;

        Ok(data)
    }

    pub unsafe fn clock_client_clk_vf_points_set_control(
        &self,
        handle: NvPhysicalGpuHandle,
        mut control: ClockClientClkVfPointsControlV1,
    ) -> anyhow::Result<()> {
        self.physical_gpu_query(
            handle,
            &mut control,
            QUERY_NVAPI_GPU_CLOCK_CLIENT_CLK_VF_POINTS_SET_CONTROL,
        )?;

        Ok(())
    }

    pub unsafe fn calculate_therm_channel_mask(
        &self,
        handle: NvPhysicalGpuHandle,
    ) -> anyhow::Result<i32> {
        let f = self.query_interface(QUERY_NVAPI_GPU_THERM_CHANNEL_GET_STATUS)?;
        let f: unsafe extern "C" fn(
            handle: NvPhysicalGpuHandle,
            sensors: &mut NvApiThermals,
        ) -> NvAPI_Status = transmute(f);

        let mut sensors = NvApiThermals {
            #[allow(clippy::cast_possible_truncation)]
            version: (mem::size_of::<NvApiThermals>() | (2 << 16)) as u32,
            mask: 1,
            values: [0; 40],
        };

        let initial_status = f(handle, &mut sensors);
        self.handle_status(initial_status)?;

        for bit in 0..32 {
            sensors.mask = 1 << bit;
            let status = f(handle, &mut sensors);
            if status != 0 {
                return Ok(sensors.mask - 1);
            }
        }

        bail!("Could not find suitable mask");
    }

    unsafe fn enum_physical_gpus(&self) -> anyhow::Result<Vec<NvPhysicalGpuHandle>> {
        let f = self.query_interface(QUERY_NVAPI_ENUM_PHYSICAL_GPUS)?;
        let f: unsafe extern "C" fn(
            handles: &mut [NvPhysicalGpuHandle; NVAPI_MAX_PHYSICAL_GPUS as usize],
            count: &mut u32,
        ) -> NvAPI_Status = transmute(f);

        let mut count = 0;
        let mut handles =
            [(ptr::null_mut() as NvPhysicalGpuHandle); NVAPI_MAX_PHYSICAL_GPUS as usize];

        let status = f(&mut handles, &mut count);
        self.handle_status(status)?;

        Ok(handles.into_iter().take(count as usize).collect())
    }

    pub fn read_vram_temps(
        &self,
        handle: NvPhysicalGpuHandle,
        vram_type: Option<&str>,
    ) -> anyhow::Result<Vec<(String, u64)>> {
        const REG_OFFSET_GDDR_TEMP_DATA_BASE: u32 = 0x009024C0;
        const REG_OFFSET_GDDR_TEMP_STATUS_BASE: u32 = 0x009024D0;
        const REG_OFFSET_GDDR_CLAMSHELL: u32 = 0x00900200;

        fn parse_temp(raw: u64) -> u64 {
            2 * raw.min(0x50) - 40
        }

        let mut temps = Vec::new();

        if let Some("GDDR6x" | "GDDR7") = vram_type {
            let is_clamshell = unsafe {
                self.read_u32_register(handle, REG_OFFSET_GDDR_CLAMSHELL)
                    .is_ok_and(|value| (value >> 22) & 1 == 1)
            };

            for partition in 0..=8 {
                let data_offset = REG_OFFSET_GDDR_TEMP_DATA_BASE + partition * 0x4000;
                let status_offset = REG_OFFSET_GDDR_TEMP_STATUS_BASE + partition * 0x4000;

                unsafe {
                    let Ok(status) = self.read_u32_register(handle, status_offset) else {
                        continue;
                    };

                    // Explicit poison data
                    if status & 0xFFFF0000 == 0xBADF0000 {
                        continue;
                    }

                    let mut i = 0;
                    'pairs: for slot_pair in [[(0x0, 24), (0x8, 26)], [(0x4, 25), (0xC, 27)]] {
                        for (slot, status_bit) in slot_pair {
                            if (status >> status_bit) & 1 == 1 {
                                let data = self.read_u32_register(handle, data_offset + slot)?;

                                if data == 0
                                    || data == u64::from(u32::MAX)
                                    || data & 0xFFFF0000 == 0xBADF0000
                                {
                                    continue;
                                }

                                let pc0 = (data >> 16) & 0xFF;
                                let pc1 = (data >> 24) & 0xFF;

                                if pc0 == 0 || pc0 == u64::from(u8::MAX) {
                                    continue;
                                }

                                let partition_letter = char::from_u32(('A' as u32) + partition)
                                    .context("Invalid partition")?;
                                let label_base = format!("{partition_letter}{i}");

                                let front_label = if is_clamshell {
                                    format!("{label_base} (Front)")
                                } else {
                                    label_base.clone()
                                };

                                temps.push((front_label, parse_temp(pc0)));

                                if is_clamshell && pc1 != 0 && pc1 != u64::from(u8::MAX) {
                                    temps.push((format!("{label_base} (Back)"), parse_temp(pc1)));
                                }

                                i += 1;

                                continue 'pairs;
                            }
                        }
                    }
                }
            }
        }

        Ok(temps)
    }

    pub unsafe fn read_blackwell_hotspot(
        &self,
        handle: NvPhysicalGpuHandle,
    ) -> anyhow::Result<u64> {
        self.read_temp_register(handle, REG_OFFSET_BLACKWELL_HOTSPOT_AGGREGATED)
    }

    unsafe fn read_temp_register(
        &self,
        handle: NvPhysicalGpuHandle,
        offset: u32,
    ) -> anyhow::Result<u64> {
        let raw = self.read_u32_register(handle, offset)?;
        let value = (raw & 0xFFFF) / 256;
        if value == 0 || value >= u64::from(u8::MAX) {
            bail!("Invalid temperature reported: {value}");
        }
        Ok(value)
    }

    unsafe fn read_u32_register(
        &self,
        handle: NvPhysicalGpuHandle,
        offset: u32,
    ) -> anyhow::Result<u64> {
        let f = self.query_interface(QUERY_NVAPI_GPU_REGISTER_OP)?;
        let f: unsafe extern "C" fn(
            handle: NvPhysicalGpuHandle,
            params: &mut NvGpuRegisterOpDataV1,
        ) -> NvAPI_Status = transmute(f);

        let mut op = [NvGpuRegisterOp::default(); 256];
        op[0] = NvGpuRegisterOp {
            offset,
            flags: (REG_OP_FLAG_READ | REG_OP_FLAG_32BIT | REG_OP_FLAG_TYPE_GLOBAL)
                .try_into()
                .unwrap(),
            ..Default::default()
        };

        let mut params = NvGpuRegisterOpDataV1 {
            op_count: 1,
            op,
            ..Default::default()
        };

        let status = f(handle, &mut params);
        self.handle_status(status)?;

        Ok(params.op[0].value)
    }

    unsafe fn query_interface(&self, id: u32) -> anyhow::Result<*const ()> {
        let query_interface = self
            .lib
            .get::<unsafe extern "C" fn(u32) -> *const ()>(QUERY_INTERFACE_FN)
            .context("Could not get main symbol")?;

        let f = query_interface(id);

        if f.is_null() {
            bail!("Got null response for query id {id:x}");
        }

        Ok(f)
    }

    unsafe fn handle_status(&self, status: NvAPI_Status) -> anyhow::Result<()> {
        if status == 0 {
            Ok(())
        } else {
            let f = self.query_interface(QUERY_NVAPI_GET_ERROR_MESSAGE)?;
            let f: unsafe extern "C" fn(
                status: NvAPI_Status,
                text: &mut [c_char; NVAPI_SHORT_STRING_MAX as usize],
            ) -> NvAPI_Status = transmute(f);

            let mut text = [0; NVAPI_SHORT_STRING_MAX as usize];
            let other_status = f(status, &mut text);
            if other_status != 0 {
                bail!(
                    "Got status {other_status:x} when fetching error message for status {status:x}"
                );
            }
            let text =
                CStr::from_bytes_until_nul(&*(ptr::from_ref::<[_]>(text.as_slice()) as *const [_]));
            bail!(
                "Got error {status:x} from NvAPI: {}",
                text.unwrap_or_default().to_string_lossy()
            );
        }
    }

    unsafe fn physical_gpu_query<T>(
        &self,
        handle: NvPhysicalGpuHandle,
        data: &mut T,
        query_id: u32,
    ) -> anyhow::Result<()> {
        let f = self.query_interface(query_id)?;
        let f: unsafe extern "C" fn(NvPhysicalGpuHandle, *mut T) -> NvAPI_Status = transmute(f);

        let status = f(handle, data);
        self.handle_status(status)?;

        Ok(())
    }

    /// Gets the value from `NvApiThermals` if possible, otherwise reads from register
    pub fn read_hotspot(
        &self,
        thermals: &NvApiThermals,
        handle: NvPhysicalGpuHandle,
        arch: Option<&DeviceArchitecture>,
    ) -> Option<i32> {
        if arch.is_some_and(|arch| arch.as_c() >= DeviceArchitecture::Blackwell.as_c()) {
            unsafe {
                self.read_blackwell_hotspot(handle)
                    .ok()
                    .and_then(|value| value.try_into().ok())
            }
        } else {
            thermals.get_value(9)
        }
    }
}

impl Drop for NvApi {
    fn drop(&mut self) {
        unsafe {
            let unload = self.query_interface(QUERY_NVAPI_UNLOAD).unwrap();
            let unload: unsafe extern "C" fn() -> NvAPI_Status = transmute(unload);
            let status = unload();
            if let Err(err) = self.handle_status(status) {
                error!("could not unload NvAPI: {err}");
            }
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct NvApiThermals {
    version: u32,
    mask: i32,
    values: [i32; 40],
}

impl NvApiThermals {
    fn get_value(&self, index: usize) -> Option<i32> {
        self.values
            .get(index)
            .map(|&value| value / 256)
            .filter(|&value| value > 0 && value < 255)
    }

    pub fn vram(&self, vram_type: Option<&str>) -> Option<i32> {
        match vram_type {
            Some("GDDR7") => self.get_value(10),
            _ => self.get_value(15),
        }
    }
}

#[repr(C)]
#[derive(Debug)]
struct ClientVoltRailsStatusV1 {
    version: NvU32,
    rsvd: [NvU8; 32],
    rails: [ClientVoltRailStatusV1; 1],
}

impl Default for ClientVoltRailsStatusV1 {
    fn default() -> Self {
        Self {
            version: make_version::<Self>(1),
            rsvd: [0; 32],
            rails: [ClientVoltRailStatusV1::default()],
        }
    }
}

#[repr(C)]
#[derive(Debug, Default)]
struct ClientVoltRailStatusV1 {
    rail_id: NvU32,
    current_voltage_uv: NvU32,
    rsvd: [NvU8; 32],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct ClientVoltRailsControlV1 {
    version: NvU32,
    percent_delta: NvU8,
    rsvd: [NvU8; 32],
}

impl Default for ClientVoltRailsControlV1 {
    fn default() -> Self {
        Self {
            version: make_version::<Self>(1),
            percent_delta: 0,
            rsvd: [0; 32],
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ClockClientClkVfPointsStatusV3 {
    pub version: NvU32,
    pub vf_points_mask: [NvU32; 8],
    pub b_vf_tuple_base_supported: NvU8,
    pub rsvd: [NvU8; 64],
    pub vf_points: [ClockClientClkVfPointStatusV3; 255],
}

impl Default for ClockClientClkVfPointsStatusV3 {
    fn default() -> Self {
        Self {
            version: make_version::<Self>(3),
            vf_points_mask: [0; 8],
            b_vf_tuple_base_supported: 0,
            rsvd: [0; 64],
            vf_points: [ClockClientClkVfPointStatusV3::default(); 255],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ClockClientClkVfPointStatusV3 {
    pub type_: NvU32,
    pub freq_khz: NvU32,
    pub voltage_uv: NvU32,
    pub vf_tuple_base: ClockClientClkVfPointTupleV1,
    pub vf_tuple_offset: ClockClientClkVfPointTupleV1,
    pub rsvd: [NvU8; 256],
}

impl Default for ClockClientClkVfPointStatusV3 {
    fn default() -> Self {
        Self {
            type_: Default::default(),
            freq_khz: Default::default(),
            voltage_uv: Default::default(),
            vf_tuple_base: ClockClientClkVfPointTupleV1::default(),
            vf_tuple_offset: ClockClientClkVfPointTupleV1::default(),
            rsvd: [0; 256],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct ClockClientClkVfPointTupleV1 {
    pub freq_khz: NvU32,
    pub voltage_uv: NvU32,
    pub rsvd: [NvU8; 32usize],
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ClockClientClkVfPointsInfoV1 {
    pub version: NvU32,
    pub vf_points_mask: [NvU32; 8],
    pub rsvd: [NvU8; 32],
    pub vf_points: [ClockClientClkVfPointInfoV1; 255],
}

impl Default for ClockClientClkVfPointsInfoV1 {
    fn default() -> Self {
        Self {
            version: make_version::<Self>(1),
            vf_points_mask: Default::default(),
            rsvd: Default::default(),
            vf_points: [ClockClientClkVfPointInfoV1::default(); 255],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct ClockClientClkVfPointInfoV1 {
    pub type_: NvU32,
    pub b_voltage_based: NvU8,
    pub rsvd: [NvU8; 16],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ClockClientClkVfPointsControlV1 {
    pub version: NvU32,
    pub vf_points_mask: [NvU32; 8],
    pub rsvd: [NvU8; 32usize],
    pub vf_points: [ClockClientClkVfPointControlV1; 255usize],
}

impl Default for ClockClientClkVfPointsControlV1 {
    fn default() -> Self {
        Self {
            version: make_version::<Self>(1),
            vf_points_mask: [0; 8],
            rsvd: [0; 32],
            vf_points: [ClockClientClkVfPointControlV1::default(); 255],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct ClockClientClkVfPointControlV1 {
    pub type_: NvU32,
    pub rsvd: [NvU8; 16usize],
    pub data: ClockClientClkVfPointControlDataV1,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union ClockClientClkVfPointControlDataV1 {
    pub prog: ClockClientClkVfPointControlProgV1,
    pub rsvd: [NvU8; 16usize],
}

impl Default for ClockClientClkVfPointControlDataV1 {
    fn default() -> Self {
        Self { rsvd: [0; 16] }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ClockClientClkVfPointControlProgV1 {
    pub freq_offset_khz: NvS32,
}

const REG_OP_FLAG_READ: c_uint = 1;
const REG_OP_FLAG_32BIT: c_uint = 4;
const REG_OP_FLAG_TYPE_GLOBAL: c_uint = 16;

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
struct NvGpuRegisterOp {
    pub flags: NvU16,
    pub status: NvU16,
    pub offset: NvU32,
    pub write_mask: NvU64,
    pub value: NvU64,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct NvGpuRegisterOpDataV1 {
    pub version: NvU32,
    pub op_count: NvU32,
    pub op: [NvGpuRegisterOp; 256usize],
}

impl Default for NvGpuRegisterOpDataV1 {
    fn default() -> Self {
        Self {
            version: make_version::<Self>(1),
            op_count: 0,
            op: [NvGpuRegisterOp::default(); 256],
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct NvGpuClockInfoV2 {
    version: NvU32,
    domain: [NvGpuClockInfoDomain; 32usize],
    extended_domain: [NvGpuClockInfoV2ExtendedDomain; 32usize],
}

impl NvGpuClockInfoV2 {
    pub fn get_domain(&self, domain: NvGpuClockDomainId) -> Option<NvGpuClockInfoDomain> {
        self.domain
            .get(domain as usize)
            .filter(|domain| domain.bitfield & 1 != 0 && domain.frequency != 0)
            .copied()
    }
}

impl Default for NvGpuClockInfoV2 {
    fn default() -> Self {
        Self {
            version: make_version::<Self>(2),
            domain: Default::default(),
            extended_domain: Default::default(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct NvGpuClockInfoDomain {
    pub frequency: NvU32,
    bitfield: NvU32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct NvGpuClockInfoV2ExtendedDomain {
    effective_frequency: NvU32,
    ratio_domain: NvGpuClockDomainId,
    ratio: NvU32,
    reserved: [NvU32; 4usize],
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NvGpuClockDomainId {
    // Gpc = 0,
    Xbar = 1,
    Sys = 2,
    Hub = 3,
    // M = 4,
    Host = 5,
    Disp = 6,
    Pclk0 = 8,
    Pclk1 = 9,
    Xclk = 11,
    Leg = 19,
    Pwr = 20,
    Msd = 21,
    Utils = 22,
    _2D = 26,
    _3D = 27,
    Host1x = 28,
    Disp0 = 29,
    Disp1 = 30,
    // Pciegen = 31,
    #[default]
    Undefined = 32,
}

impl NvGpuClockDomainId {
    /// Excludes the domains already reported through NVML
    pub const EXTRA: [Self; 17] = [
        // Self::Gpc,
        Self::Xbar,
        Self::Sys,
        Self::Hub,
        // Self::M,
        Self::Host,
        Self::Disp,
        Self::Pclk0,
        Self::Pclk1,
        Self::Xclk,
        Self::Leg,
        Self::Pwr,
        Self::Msd,
        Self::Utils,
        Self::_2D,
        Self::_3D,
        Self::Host1x,
        Self::Disp0,
        Self::Disp1,
    ];

    pub fn into_str(self) -> &'static str {
        match self {
            // Self::Gpc => "GPC",
            Self::Xbar => "XBAR",
            Self::Sys => "SYS",
            Self::Hub => "HUB",
            // Self::M => "Memory",
            Self::Host => "Host",
            Self::Disp => "Display",
            Self::Pclk0 => "Pixel Clock 0",
            Self::Pclk1 => "Pixel Clock 1",
            Self::Xclk => "External Clock",
            Self::Leg => "Legacy",
            Self::Pwr => "Power",
            Self::Msd => "MSD",
            Self::Utils => "Utils",
            Self::_2D => "2D",
            Self::_3D => "3D",
            Self::Host1x => "Host1x",
            Self::Disp0 => "Display 0",
            Self::Disp1 => "Display 1",
            // Self::Pciegen => "PCIe",
            Self::Undefined => "Undefined",
        }
    }
}

impl std::fmt::Display for NvGpuClockDomainId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.into_str().fmt(f)
    }
}

#[allow(clippy::cast_possible_truncation)]
const fn make_version<T>(version: usize) -> u32 {
    (mem::size_of::<T>() | (version << 16)) as u32
}
