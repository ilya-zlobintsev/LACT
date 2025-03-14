use std::{
    ffi::c_void,
    fs::File,
    mem,
    os::fd::{AsRawFd, RawFd},
    ptr,
};

use crate::bindings::nvidia::{
    NvHandle, NV0080_ALLOC_PARAMETERS, NV01_DEVICE_0, NV2080_ALLOC_PARAMETERS,
    NV2080_CTRL_CMD_GR_GET_ROP_INFO, NV2080_CTRL_GR_GET_ROP_INFO_PARAMS, NV20_SUBDEVICE_0,
    NVOS21_PARAMETERS, NVOS54_PARAMETERS, NVOS64_PARAMETERS, NV_ESC_REGISTER_FD, NV_ESC_RM_ALLOC,
    NV_ESC_RM_CONTROL, NV_IOCTL_MAGIC,
};
use anyhow::{bail, Context};
use lact_schema::NvidiaRopInfo;
use nix::ioctl_readwrite;

pub struct DriverHandle {
    nvidiactl_fd: File,
    #[allow(dead_code)]
    device_fd: File,

    client_handle: NvHandle,
    #[allow(dead_code)]
    device_handle: NvHandle,
    subdevice_handle: NvHandle,
}

impl DriverHandle {
    pub fn open(minor_number: u32) -> anyhow::Result<Self> {
        let nvidiactl_fd = File::options()
            .read(true)
            .write(true)
            .open("/dev/nvidiactl")
            .context("Could not open nvidiactl")?;

        let client_handle: NvHandle = unsafe {
            let mut client_request: NVOS21_PARAMETERS = mem::zeroed();
            rm_alloc_nvos21(nvidiactl_fd.as_raw_fd(), &mut client_request)?;
            client_request.hObjectNew
        };

        let device_fd = File::options()
            .read(true)
            .write(true)
            .open(format!("/dev/nvidia{minor_number}"))
            .context("Could not open nvidia device")?;

        let device_handle: NvHandle = unsafe {
            register_fd(device_fd.as_raw_fd(), &mut nvidiactl_fd.as_raw_fd())?;

            let mut alloc_params: NV0080_ALLOC_PARAMETERS = mem::zeroed();
            let mut request = NVOS64_PARAMETERS {
                hRoot: client_handle,
                hObjectParent: client_handle,
                hObjectNew: 0,
                hClass: NV01_DEVICE_0,
                pAllocParms: ptr::from_mut(&mut alloc_params).cast::<c_void>(),
                pRightsRequested: ptr::null_mut(),
                paramsSize: 0,
                flags: 0,
                status: 0,
            };

            rm_alloc_nvos64(nvidiactl_fd.as_raw_fd(), &mut request)?;

            if request.status != 0 {
                bail!(
                    "Got error status {} on Nvidia device handle creation",
                    request.status
                );
            }

            request.hObjectNew
        };

        let subdevice_handle: NvHandle = unsafe {
            let mut alloc_params: NV2080_ALLOC_PARAMETERS = mem::zeroed();

            let mut request = NVOS64_PARAMETERS {
                hRoot: client_handle,
                hObjectParent: device_handle,
                hObjectNew: 0,
                hClass: NV20_SUBDEVICE_0,
                pAllocParms: ptr::from_mut(&mut alloc_params).cast(),
                pRightsRequested: ptr::null_mut(),
                paramsSize: 0,
                flags: 0,
                status: 0,
            };

            rm_alloc_nvos64(nvidiactl_fd.as_raw_fd(), &mut request)?;

            if request.status != 0 {
                bail!(
                    "Got error status {} on Nvidia subdevice handle creation",
                    request.status
                );
            }

            request.hObjectNew
        };

        Ok(Self {
            nvidiactl_fd,
            device_fd,
            client_handle,
            device_handle,
            subdevice_handle,
        })
    }

    pub fn get_rop_info(&self) -> anyhow::Result<NvidiaRopInfo> {
        unsafe {
            let mut params: NV2080_CTRL_GR_GET_ROP_INFO_PARAMS = mem::zeroed();

            let mut request = NVOS54_PARAMETERS {
                hClient: self.client_handle,
                hObject: self.subdevice_handle,
                cmd: NV2080_CTRL_CMD_GR_GET_ROP_INFO,
                flags: 0,
                params: ptr::from_mut(&mut params).cast(),
                paramsSize: mem::size_of::<NV2080_CTRL_GR_GET_ROP_INFO_PARAMS>()
                    .try_into()
                    .unwrap(),
                status: 0,
            };

            rm_control_nvos54(self.nvidiactl_fd.as_raw_fd(), &mut request)?;

            if request.status != 0 {
                bail!("ROP request failed with status {}", request.status);
            }

            Ok(NvidiaRopInfo {
                unit_count: params.ropUnitCount,
                operations_factor: params.ropOperationsFactor,
                operations_count: params.ropOperationsCount,
            })
        }
    }
}

ioctl_readwrite!(
    rm_alloc_nvos21,
    NV_IOCTL_MAGIC,
    NV_ESC_RM_ALLOC,
    NVOS21_PARAMETERS
);

ioctl_readwrite!(
    rm_alloc_nvos64,
    NV_IOCTL_MAGIC,
    NV_ESC_RM_ALLOC,
    NVOS64_PARAMETERS
);

ioctl_readwrite!(register_fd, NV_IOCTL_MAGIC, NV_ESC_REGISTER_FD, RawFd);

ioctl_readwrite!(
    rm_control_nvos54,
    NV_IOCTL_MAGIC,
    NV_ESC_RM_CONTROL,
    NVOS54_PARAMETERS
);
