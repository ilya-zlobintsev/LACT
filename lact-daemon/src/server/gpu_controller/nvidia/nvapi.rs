#![cfg_attr(test, allow(unused))]
#![allow(clippy::unreadable_literal)]
use crate::bindings::nvidia::{
    NvAPI_Status, NvPhysicalGpuHandle, NVAPI_MAX_PHYSICAL_GPUS, NVAPI_SHORT_STRING_MAX,
};
use anyhow::{bail, Context};
use std::{
    ffi::{c_char, CStr},
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
const QUERY_NVAPI_THERMALS: u32 = 0x65fe3aad; // Undocumented call
const QUERY_NVAPI_VOLTAGE: u32 = 0x465f9bcf; // Undocumented call

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

            Ok(None)
        }
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
            version: (mem::size_of::<NvApiThermals>() | (2 << 16)) as u32,
            mask,
            values: [0; 40],
        };

        let status = f(handle, &mut sensors);
        self.handle_status(status)?;

        Ok(sensors)
    }

    pub unsafe fn get_voltage(&self, handle: NvPhysicalGpuHandle) -> anyhow::Result<u32> {
        let f = self.query_interface(QUERY_NVAPI_VOLTAGE)?;
        let f: unsafe extern "C" fn(
            handle: NvPhysicalGpuHandle,
            data: &mut NvApiVoltage,
        ) -> NvAPI_Status = transmute(f);

        let mut data = NvApiVoltage {
            #[allow(clippy::cast_possible_truncation)]
            version: (mem::size_of::<NvApiVoltage>() | (1 << 16)) as u32,
            flags: 0,
            padding_1: [0; 8],
            value_uv: 0,
            padding_2: [0; 8],
        };
        let status = f(handle, &mut data);
        self.handle_status(status)?;

        Ok(data.value_uv)
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
            // let text = CStr::from_bytes_until_nul(transmute::<&[i8], &[u8]>(text.as_slice()));
            let text = CStr::from_bytes_until_nul(
                &*(ptr::from_ref::<[i8]>(text.as_slice()) as *const [u8]),
            );
            bail!(
                "Got error {status:x} from NvAPI: {}",
                text.unwrap_or_default().to_string_lossy()
            );
        }
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
    pub version: u32,
    pub mask: i32,
    pub values: [i32; 40],
}

impl NvApiThermals {
    pub fn hotspot(&self) -> i32 {
        self.values[9] / 256
    }

    pub fn vram(&self) -> i32 {
        self.values[15] / 256
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
