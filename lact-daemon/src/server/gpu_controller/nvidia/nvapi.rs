#![cfg_attr(test, allow(unused))]
#![allow(
    clippy::unreadable_literal,
    unsafe_op_in_unsafe_fn,
    clippy::large_stack_arrays
)]

use crate::bindings::nvidia::{
    NVAPI_MAX_PHYSICAL_GPUS, NVAPI_SHORT_STRING_MAX, NvAPI_Status, NvPhysicalGpuHandle, NvS32,
    NvU8, NvU32,
};
use anyhow::{Context, bail};
use std::{
    ffi::{CStr, c_char},
    mem::{self, transmute},
    ptr,
};

const LIBARY_NAME: &str = "libnvidia-api.so.1";
const QUERY_INTERFACE_FN: &[u8] = b"nvapi_QueryInterface\0";

const QUERY_NVAPI_INITIALIZE: u32 = 0x0150e828;
const QUERY_NVAPI_UNLOAD: u32 = 0xd22bdd7e;
const QUERY_NVAPI_ENUM_PHYSICAL_GPUS: u32 = 0xe5ac921f;
const QUERY_NVAPI_GET_BUS_ID: u32 = 0x1be0b8e5;
const QUERY_NVAPI_GET_ERROR_MESSAGE: u32 = 0x6c2d048c;
// Undocumented calls
const QUERY_NVAPI_THERMALS: u32 = 0x65fe3aad;
const QUERY_NVAPI_VOLTAGE: u32 = 0x465f9bcf;
const QUERY_NVAPI_GPU_CLOCK_CLIENT_CLK_VF_POINTS_GET_STATUS: u32 = 0x21537ad4;
const QUERY_NVAPI_GPU_CLOCK_CLIENT_CLK_VF_POINTS_GET_INFO: u32 = 0x507b4b59;
const QUERY_NVAPI_GPU_CLOCK_CLIENT_CLK_VF_POINTS_SET_CONTROL: u32 = 0x733e009;
const QUERY_NVAPI_GPU_CLOCK_CLIENT_CLK_VF_POINTS_GET_CONTROL: u32 = 0x23f1b133;

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
                let f = self.query_interface(QUERY_NVAPI_GET_BUS_ID)?;
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

    pub unsafe fn get_thermals(
        &self,
        handle: NvPhysicalGpuHandle,
        mask: i32,
    ) -> anyhow::Result<NvApiThermals> {
        let f = self.query_interface(QUERY_NVAPI_THERMALS)?;
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

    pub unsafe fn get_voltage(&self, handle: NvPhysicalGpuHandle) -> anyhow::Result<u32> {
        let mut data = NvApiVoltage {
            #[allow(clippy::cast_possible_truncation)]
            version: make_version::<NvApiVoltage>(1),
            flags: 0,
            padding_1: [0; 8],
            value_uv: 0,
            padding_2: [0; 8],
        };

        self.physical_gpu_query(handle, &mut data, QUERY_NVAPI_VOLTAGE)?;

        Ok(data.value_uv)
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

    pub unsafe fn clock_client_clk_vf_get_control(
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

    pub unsafe fn clock_client_clk_vf_set_control(
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

    pub unsafe fn calculate_thermals_mask(
        &self,
        handle: NvPhysicalGpuHandle,
    ) -> anyhow::Result<i32> {
        let f = self.query_interface(QUERY_NVAPI_THERMALS)?;
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
}

impl Drop for NvApi {
    fn drop(&mut self) {
        unsafe {
            let unload = self.query_interface(QUERY_NVAPI_UNLOAD).unwrap();
            let unload: unsafe extern "C" fn() -> NvAPI_Status = transmute(unload);
            unload();
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

    pub fn hotspot(&self) -> Option<i32> {
        self.get_value(9)
    }

    pub fn vram(&self) -> Option<i32> {
        self.get_value(15)
    }
}

#[repr(C)]
#[derive(Debug)]
struct NvApiVoltage {
    version: u32,
    flags: u32,
    padding_1: [u32; 8],
    value_uv: u32,
    padding_2: [u32; 8],
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

#[allow(clippy::cast_possible_truncation)]
const fn make_version<T>(version: usize) -> u32 {
    (mem::size_of::<T>() | (version << 16)) as u32
}
