use std::{
    fs::File,
    mem,
    os::fd::{AsRawFd, OwnedFd, RawFd},
    ptr, slice,
};

use crate::bindings::nvidia::{
    DRM_COMMAND_BASE, DRM_IOCTL_BASE, DRM_NVIDIA_GET_DPY_ID_FOR_CONNECTOR_ID, NV_ESC_REGISTER_FD,
    NV_ESC_RM_ALLOC, NV_ESC_RM_CONTROL, NV_IOCTL_MAGIC, NV01_DEVICE_0, NV20_SUBDEVICE_0,
    NV0080_ALLOC_PARAMETERS, NV2080_ALLOC_PARAMETERS, NV2080_CTRL_CMD_FB_GET_INFO,
    NV2080_CTRL_CMD_GR_GET_GLOBAL_SM_ORDER, NV2080_CTRL_CMD_GR_GET_ROP_INFO,
    NV2080_CTRL_FB_GET_INFO_PARAMS, NV2080_CTRL_FB_INFO, NV2080_CTRL_FB_INFO_INDEX_BUS_WIDTH,
    NV2080_CTRL_FB_INFO_INDEX_L2CACHE_SIZE, NV2080_CTRL_FB_INFO_INDEX_MEMORYINFO_VENDOR_ID,
    NV2080_CTRL_FB_INFO_INDEX_RAM_TYPE, NV2080_CTRL_FB_INFO_MEMORYINFO_VENDOR_ID_ELPIDA,
    NV2080_CTRL_FB_INFO_MEMORYINFO_VENDOR_ID_ESMT, NV2080_CTRL_FB_INFO_MEMORYINFO_VENDOR_ID_ETRON,
    NV2080_CTRL_FB_INFO_MEMORYINFO_VENDOR_ID_HYNIX,
    NV2080_CTRL_FB_INFO_MEMORYINFO_VENDOR_ID_MICRON,
    NV2080_CTRL_FB_INFO_MEMORYINFO_VENDOR_ID_MOSEL, NV2080_CTRL_FB_INFO_MEMORYINFO_VENDOR_ID_NANYA,
    NV2080_CTRL_FB_INFO_MEMORYINFO_VENDOR_ID_QIMONDA,
    NV2080_CTRL_FB_INFO_MEMORYINFO_VENDOR_ID_SAMSUNG,
    NV2080_CTRL_FB_INFO_MEMORYINFO_VENDOR_ID_UNKNOWN,
    NV2080_CTRL_FB_INFO_MEMORYINFO_VENDOR_ID_WINBOND, NV2080_CTRL_FB_INFO_RAM_TYPE_DDR1,
    NV2080_CTRL_FB_INFO_RAM_TYPE_DDR2, NV2080_CTRL_FB_INFO_RAM_TYPE_DDR3,
    NV2080_CTRL_FB_INFO_RAM_TYPE_GDDR2, NV2080_CTRL_FB_INFO_RAM_TYPE_GDDR3,
    NV2080_CTRL_FB_INFO_RAM_TYPE_GDDR4, NV2080_CTRL_FB_INFO_RAM_TYPE_GDDR5,
    NV2080_CTRL_FB_INFO_RAM_TYPE_GDDR5X, NV2080_CTRL_FB_INFO_RAM_TYPE_GDDR6,
    NV2080_CTRL_FB_INFO_RAM_TYPE_GDDR6X, NV2080_CTRL_FB_INFO_RAM_TYPE_GDDR7,
    NV2080_CTRL_FB_INFO_RAM_TYPE_HBM1, NV2080_CTRL_FB_INFO_RAM_TYPE_HBM2,
    NV2080_CTRL_FB_INFO_RAM_TYPE_HBM3, NV2080_CTRL_FB_INFO_RAM_TYPE_HBM4,
    NV2080_CTRL_FB_INFO_RAM_TYPE_LPDDR2, NV2080_CTRL_FB_INFO_RAM_TYPE_LPDDR4,
    NV2080_CTRL_FB_INFO_RAM_TYPE_LPDDR5, NV2080_CTRL_FB_INFO_RAM_TYPE_SDDR4,
    NV2080_CTRL_FB_INFO_RAM_TYPE_SDRAM, NV2080_CTRL_FB_INFO_RAM_TYPE_UNKNOWN,
    NV2080_CTRL_GR_GET_GLOBAL_SM_ORDER_PARAMS, NV2080_CTRL_GR_GET_ROP_INFO_PARAMS,
    NVOS21_PARAMETERS, NVOS54_PARAMETERS, NVOS64_PARAMETERS, NvHandle, NvU32,
    drm_nvidia_get_dpy_id_for_connector_id_params,
};
#[cfg(feature = "display-info")]
use crate::bindings::nvidia::{
    NV04_DISPLAY_COMMON, NV0073_CTRL_CMD_DP_GET_LINK_CONFIG, NV0073_CTRL_DP_GET_LINK_CONFIG_PARAMS,
};
use anyhow::{Context, bail, ensure};
use lact_schema::RopInfo;
use nix::ioctl_readwrite;

pub struct DriverHandle {
    nvidiactl_fd: OwnedFd,
    #[allow(dead_code)]
    device_fd: OwnedFd,

    client_handle: NvHandle,
    #[allow(dead_code)]
    device_handle: NvHandle,
    subdevice_handle: NvHandle,
    #[cfg(feature = "display-info")]
    display_handle: Option<NvHandle>,
}

impl DriverHandle {
    pub fn open(minor_number: u32) -> anyhow::Result<Self> {
        let nvidiactl_fd: OwnedFd = File::options()
            .read(true)
            .write(true)
            .open("/dev/nvidiactl")
            .context("Could not open nvidiactl")?
            .into();

        let client_handle: NvHandle = unsafe {
            let mut client_request: NVOS21_PARAMETERS = mem::zeroed();
            rm_alloc_nvos21(nvidiactl_fd.as_raw_fd(), &raw mut client_request)?;
            client_request.hObjectNew
        };

        let device_fd: OwnedFd = File::options()
            .read(true)
            .write(true)
            .open(format!("/dev/nvidia{minor_number}"))
            .context("Could not open nvidia device")?
            .into();

        let device_handle: NvHandle = unsafe {
            register_fd(device_fd.as_raw_fd(), &mut nvidiactl_fd.as_raw_fd())?;

            let mut alloc_params: NV0080_ALLOC_PARAMETERS = mem::zeroed();
            alloc_params.deviceId = minor_number;

            alloc_object(
                client_handle,
                client_handle,
                NV01_DEVICE_0,
                Some(&mut alloc_params),
                nvidiactl_fd.as_raw_fd(),
            )?
        };

        let subdevice_handle: NvHandle = unsafe {
            let mut alloc_params: NV2080_ALLOC_PARAMETERS = mem::zeroed();

            alloc_object(
                client_handle,
                device_handle,
                NV20_SUBDEVICE_0,
                Some(&mut alloc_params),
                nvidiactl_fd.as_raw_fd(),
            )?
        };

        #[cfg(feature = "display-info")]
        let display_handle = unsafe {
            use tracing::warn;

            alloc_object::<()>(
                client_handle,
                device_handle,
                NV04_DISPLAY_COMMON,
                None,
                nvidiactl_fd.as_raw_fd(),
            )
            .inspect_err(|err| {
                warn!("could not allocate display handle: {err:#}");
            })
            .ok()
        };

        Ok(Self {
            nvidiactl_fd,
            device_fd,
            client_handle,
            device_handle,
            subdevice_handle,
            #[cfg(feature = "display-info")]
            display_handle,
        })
    }

    pub fn get_rop_info(&self) -> anyhow::Result<RopInfo> {
        unsafe {
            let mut params: NV2080_CTRL_GR_GET_ROP_INFO_PARAMS = mem::zeroed();
            self.query_rm_control(NV2080_CTRL_CMD_GR_GET_ROP_INFO, &mut params)?;

            Ok(RopInfo {
                unit_count: params.ropUnitCount,
                operations_factor: params.ropOperationsFactor,
                operations_count: params.ropOperationsCount,
            })
        }
    }

    pub fn get_sm_count(&self) -> anyhow::Result<u32> {
        unsafe {
            let mut params: NV2080_CTRL_GR_GET_GLOBAL_SM_ORDER_PARAMS = mem::zeroed();
            self.query_rm_control(NV2080_CTRL_CMD_GR_GET_GLOBAL_SM_ORDER, &mut params)?;
            Ok(u32::from(params.numSm))
        }
    }

    pub fn get_ram_type(&self) -> anyhow::Result<&'static str> {
        let value = self.get_fb_info(NV2080_CTRL_FB_INFO_INDEX_RAM_TYPE)?;
        let name = match value {
            NV2080_CTRL_FB_INFO_RAM_TYPE_GDDR2 => "GDDR2",
            NV2080_CTRL_FB_INFO_RAM_TYPE_GDDR3 => "GDDR3",
            NV2080_CTRL_FB_INFO_RAM_TYPE_GDDR4 => "GDDR4",
            NV2080_CTRL_FB_INFO_RAM_TYPE_GDDR5 => "GDDR5",
            NV2080_CTRL_FB_INFO_RAM_TYPE_GDDR5X => "GDDR5X",
            NV2080_CTRL_FB_INFO_RAM_TYPE_GDDR6 => "GDDR6",
            NV2080_CTRL_FB_INFO_RAM_TYPE_GDDR6X => "GDDR6x",
            NV2080_CTRL_FB_INFO_RAM_TYPE_GDDR7 => "GDDR7",

            NV2080_CTRL_FB_INFO_RAM_TYPE_HBM1 => "HBM1",
            NV2080_CTRL_FB_INFO_RAM_TYPE_HBM2 => "HBM2",
            NV2080_CTRL_FB_INFO_RAM_TYPE_HBM3 => "HBM3",
            NV2080_CTRL_FB_INFO_RAM_TYPE_HBM4 => "HBM4",

            NV2080_CTRL_FB_INFO_RAM_TYPE_DDR1 => "DDR1",
            NV2080_CTRL_FB_INFO_RAM_TYPE_DDR2 => "DDR2",
            NV2080_CTRL_FB_INFO_RAM_TYPE_DDR3 => "DDR3",

            NV2080_CTRL_FB_INFO_RAM_TYPE_LPDDR2 => "LPDDR2",
            NV2080_CTRL_FB_INFO_RAM_TYPE_LPDDR4 => "LPDDR4",
            NV2080_CTRL_FB_INFO_RAM_TYPE_LPDDR5 => "LPDDR5",

            NV2080_CTRL_FB_INFO_RAM_TYPE_SDDR4 => "SDDR4",
            NV2080_CTRL_FB_INFO_RAM_TYPE_SDRAM => "SDRAM",

            NV2080_CTRL_FB_INFO_RAM_TYPE_UNKNOWN => "Unknown",
            _ => "Unrecognized",
        };
        Ok(name)
    }

    pub fn get_bus_width(&self) -> anyhow::Result<u32> {
        self.get_fb_info(NV2080_CTRL_FB_INFO_INDEX_BUS_WIDTH)
    }

    pub fn get_ram_vendor(&self) -> anyhow::Result<&'static str> {
        let value = self.get_fb_info(NV2080_CTRL_FB_INFO_INDEX_MEMORYINFO_VENDOR_ID)?;
        let name = match value {
            NV2080_CTRL_FB_INFO_MEMORYINFO_VENDOR_ID_SAMSUNG => "Samsung",
            NV2080_CTRL_FB_INFO_MEMORYINFO_VENDOR_ID_QIMONDA => "Qimonda",
            NV2080_CTRL_FB_INFO_MEMORYINFO_VENDOR_ID_ELPIDA => "Elpida",
            NV2080_CTRL_FB_INFO_MEMORYINFO_VENDOR_ID_ETRON => "Etron",
            NV2080_CTRL_FB_INFO_MEMORYINFO_VENDOR_ID_NANYA => "Nanya",
            NV2080_CTRL_FB_INFO_MEMORYINFO_VENDOR_ID_HYNIX => "SK Hynix",
            NV2080_CTRL_FB_INFO_MEMORYINFO_VENDOR_ID_MOSEL => "Mosel",
            NV2080_CTRL_FB_INFO_MEMORYINFO_VENDOR_ID_WINBOND => "Winbond",
            NV2080_CTRL_FB_INFO_MEMORYINFO_VENDOR_ID_ESMT => "ESMT",
            NV2080_CTRL_FB_INFO_MEMORYINFO_VENDOR_ID_MICRON => "Micron",
            NV2080_CTRL_FB_INFO_MEMORYINFO_VENDOR_ID_UNKNOWN => "Unknown",
            _ => "Unrecognized",
        };
        Ok(name)
    }

    pub fn get_l2_cache_size(&self) -> anyhow::Result<u32> {
        self.get_fb_info(NV2080_CTRL_FB_INFO_INDEX_L2CACHE_SIZE)
    }

    #[cfg(feature = "display-info")]
    pub fn get_dp_link_config(
        &self,
        display_id: u32,
    ) -> anyhow::Result<NV0073_CTRL_DP_GET_LINK_CONFIG_PARAMS> {
        let display_obj = self
            .display_handle
            .context("Display object not available")?;

        let mut params = NV0073_CTRL_DP_GET_LINK_CONFIG_PARAMS {
            subDeviceInstance: 0,
            displayId: display_id,
            laneCount: 0,
            linkBW: 0,
            dp2LinkBW: 0,
            bFECEnabled: 0,
        };

        unsafe {
            self.query_rm_control_on_object(
                NV0073_CTRL_CMD_DP_GET_LINK_CONFIG,
                display_obj,
                &mut params,
            )?;
        }

        Ok(params)
    }

    /// Reads the offset state of every clock domain that can be adjusted.
    ///
    /// Domains that report a zero offset range are skipped, as they are read-only.
    pub fn get_clock_domains(&self) -> anyhow::Result<Vec<ClockDomainState>> {
        let mut info: ClkDomainsInfoParams = unsafe { mem::zeroed() };
        unsafe {
            self.query_rm_control(NV2080_CTRL_CMD_CLK_CLK_DOMAINS_GET_INFO, &mut info)?;
        }

        let control = self.get_clock_domains_control(info.domain_mask)?;

        let mut domains = Vec::new();
        for index in 0..CLK_DOMAIN_COUNT {
            if info.domain_mask & (1 << index) == 0 {
                continue;
            }

            let entry = &info.domains[index];
            // A single set bit identifies the domain; anything else is not a domain
            // we know how to name.
            if !entry.domain_bit.is_power_of_two() {
                continue;
            }
            let Some((min_offset_mhz, max_offset_mhz)) = entry.offset_range_mhz() else {
                continue;
            };

            domains.push(ClockDomainState {
                domain: entry.domain_bit.trailing_zeros(),
                freq_offset_khz: control.domains[index].freq_offset_khz,
                msvdd_offset_uv: control.domains[index].voltage_offsets_uv[MSVDD_RAIL_INDEX],
                min_offset_mhz,
                max_offset_mhz,
            });
        }

        Ok(domains)
    }

    /// Applies frequency and MSVDD offsets to the given clock domains.
    ///
    /// The RM control writes the whole domain group in one transaction, so the
    /// current block is read back first and domains that were not requested keep
    /// their existing values.
    pub fn set_clock_domain_offsets(&self, offsets: &[ClockDomainOffset]) -> anyhow::Result<()> {
        let mut info: ClkDomainsInfoParams = unsafe { mem::zeroed() };
        unsafe {
            self.query_rm_control(NV2080_CTRL_CMD_CLK_CLK_DOMAINS_GET_INFO, &mut info)?;
        }

        let mut control = self.get_clock_domains_control(info.domain_mask)?;

        for offset in offsets {
            let index = (0..CLK_DOMAIN_COUNT)
                .filter(|index| info.domain_mask & (1 << index) != 0)
                .find(|index| info.domains[*index].domain_bit == 1 << offset.domain)
                .with_context(|| format!("GPU has no clock domain {}", offset.domain))?;

            let (min_mhz, max_mhz) = info.domains[index]
                .offset_range_mhz()
                .with_context(|| format!("Clock domain {} is not adjustable", offset.domain))?;

            let requested_mhz = offset.freq_offset_khz / 1000;
            if requested_mhz < min_mhz || requested_mhz > max_mhz {
                bail!(
                    "Clock offset {requested_mhz}MHz is outside of the range \
                     {min_mhz}..{max_mhz} of domain {}",
                    offset.domain,
                );
            }

            let entry = &mut control.domains[index];
            entry.freq_offset_mode = FREQ_OFFSET_MODE_KHZ;
            entry.freq_offset_khz = offset.freq_offset_khz;
            entry.voltage_offsets_uv[MSVDD_RAIL_INDEX] = offset.msvdd_offset_uv;
        }

        unsafe {
            self.query_rm_control(NV2080_CTRL_CMD_CLK_CLK_DOMAINS_SET_CONTROL, &mut control)
                .context("Could not apply clock domain offsets")?;
        }

        Ok(())
    }

    /// Reads the GPC to XBAR propagation ratio, if this GPU has one.
    ///
    /// Returns `None` on cards where the relationship carries no ratio, which is
    /// every generation before Blackwell.
    pub fn get_clock_propagation_ratio(&self) -> anyhow::Result<Option<ClockPropagationRatio>> {
        let info = self.get_clock_prop_rels_info()?;
        let Some(index) = self.find_gpc_to_xbar_relation(&info) else {
            return Ok(None);
        };

        let control = self.get_clock_prop_rels_control(&info)?;
        Ok(Some(ClockPropagationRatio {
            current: ratio_from_fixed(control.relations[index].ratio),
            // The info response keeps reporting the factory ratio after a write,
            // which is the only way back to stock once the control has changed.
            default: ratio_from_fixed(info.relations[index].ratio),
        }))
    }

    /// Sets the GPC to XBAR propagation ratio.
    ///
    /// The whole relationship group is written in one transaction, so the current
    /// block is kept intact and only the one ratio field is replaced. The write is
    /// read back and the original block restored if anything but that field moved,
    /// since the rest of the block describes topology this does not understand.
    pub fn set_clock_propagation_ratio(&self, ratio: f64) -> anyhow::Result<()> {
        ensure!(ratio > 0.0, "Clock propagation ratio must be positive");

        let info = self.get_clock_prop_rels_info()?;
        let index = self
            .find_gpc_to_xbar_relation(&info)
            .context("GPU has no adjustable GPC to XBAR clock ratio")?;

        let preimage = self.get_clock_prop_rels_control(&info)?;
        let mut control = preimage;
        control.relations[index].ratio = ratio_to_fixed(ratio);

        unsafe {
            self.query_rm_control(NV2080_CTRL_CMD_CLK_TOP_PROP_RELS_SET_CONTROL, &mut control)
                .context("Could not apply clock propagation ratio")?;
        }

        let applied = self.get_clock_prop_rels_control(&info)?;
        if applied.relations[index].ratio != control.relations[index].ratio {
            let mut restore = preimage;
            unsafe {
                let _ = self
                    .query_rm_control(NV2080_CTRL_CMD_CLK_TOP_PROP_RELS_SET_CONTROL, &mut restore);
            }
            bail!(
                "Clock propagation ratio was not applied: requested {ratio}, driver reports {}",
                ratio_from_fixed(applied.relations[index].ratio),
            );
        }

        Ok(())
    }

    /// The first relationship that propagates GPC to XBAR through a ratio.
    ///
    /// Several such relationships can exist (a Blackwell card reports five, with
    /// differing factory ratios) and what selects between them is not known, so
    /// only the first is ever touched.
    fn find_gpc_to_xbar_relation(&self, info: &ClkPropRelsInfoParams) -> Option<usize> {
        let domains = self.get_clock_domains().ok()?;
        let index_of = |domain: u32| domains.iter().position(|state| state.domain == domain);

        (0..CLK_PROP_REL_COUNT)
            .filter(|index| info.relation_mask & (1 << index) != 0)
            .find(|index| {
                let relation = &info.relations[*index];
                relation.rel_type == CLK_PROP_REL_TYPE_RATIO
                    && Some(usize::from(relation.source_index)) == index_of(NV_CLK_DOMAIN_GPC)
                    && Some(usize::from(relation.dest_index)) == index_of(NV_CLK_DOMAIN_XBAR)
            })
    }

    fn get_clock_prop_rels_info(&self) -> anyhow::Result<ClkPropRelsInfoParams> {
        let mut info: ClkPropRelsInfoParams = unsafe { mem::zeroed() };
        unsafe {
            self.query_rm_control(NV2080_CTRL_CMD_CLK_TOP_PROP_RELS_GET_INFO, &mut info)?;
        }
        Ok(info)
    }

    fn get_clock_prop_rels_control(
        &self,
        info: &ClkPropRelsInfoParams,
    ) -> anyhow::Result<ClkPropRelsControlParams> {
        let mut control: ClkPropRelsControlParams = unsafe { mem::zeroed() };
        // The request header is the start of the info response.
        control.header.copy_from_slice(unsafe {
            slice::from_raw_parts(ptr::from_ref(info).cast::<u8>(), CLK_PROP_RELS_HEADER_LEN)
        });
        unsafe {
            self.query_rm_control(NV2080_CTRL_CMD_CLK_TOP_PROP_RELS_GET_CONTROL, &mut control)?;
        }
        Ok(control)
    }

    fn get_clock_domains_control(
        &self,
        domain_mask: u32,
    ) -> anyhow::Result<ClkDomainsControlParams> {
        let mut control: ClkDomainsControlParams = unsafe { mem::zeroed() };
        // The driver rejects a mask with bits set for domains that do not exist,
        // so this has to be the mask reported by GET_INFO.
        control.domain_mask = domain_mask;
        unsafe {
            self.query_rm_control(NV2080_CTRL_CMD_CLK_CLK_DOMAINS_GET_CONTROL, &mut control)?;
        }
        Ok(control)
    }

    fn get_fb_info(&self, stat_index: u32) -> anyhow::Result<u32> {
        let mut info_list = vec![NV2080_CTRL_FB_INFO {
            index: stat_index,
            data: 0,
        }];
        let mut params = NV2080_CTRL_FB_GET_INFO_PARAMS {
            fbInfoListSize: u32::try_from(info_list.len()).unwrap(),
            fbInfoList: info_list.as_mut_ptr().cast(),
        };

        unsafe {
            self.query_rm_control(NV2080_CTRL_CMD_FB_GET_INFO, &mut params)?;
        }

        Ok(info_list[0].data)
    }

    unsafe fn query_rm_control<T: Copy>(&self, cmd: u32, params: &mut T) -> anyhow::Result<()> {
        unsafe { self.query_rm_control_on_object(cmd, self.subdevice_handle, params) }
    }

    unsafe fn query_rm_control_on_object<T: Copy>(
        &self,
        cmd: u32,
        object: u32,
        params: &mut T,
    ) -> anyhow::Result<()> {
        let mut request = NVOS54_PARAMETERS {
            hClient: self.client_handle,
            hObject: object,
            cmd,
            flags: 0,
            params: ptr::from_mut(params).cast(),
            paramsSize: mem::size_of::<T>().try_into().unwrap(),
            status: 0,
        };
        unsafe {
            rm_control_nvos54(self.nvidiactl_fd.as_raw_fd(), &raw mut request)?;
        }

        if request.status != 0 {
            bail!("Nvidia request failed with status {:x}", request.status);
        }

        Ok(())
    }
}

/// Undocumented RM controls for the clock domain board object group.
///
/// These are the same commands that `libnvidia-api.so.1` issues internally for its
/// `ClockClientClkDomains*` entry points. They are not part of the public headers,
/// so the parameter layouts below were recovered from the driver's own translation
/// code and verified against an RTX 5090 on driver 610.57.04. Treat them as
/// branch-specific: every field is validated against `GET_INFO` before use, and a
/// layout change should surface as a failed control rather than a bad write.
const NV2080_CTRL_CMD_CLK_CLK_DOMAINS_GET_INFO: u32 = 0x2080_9019;
const NV2080_CTRL_CMD_CLK_CLK_DOMAINS_GET_CONTROL: u32 = 0x2080_901b;
const NV2080_CTRL_CMD_CLK_CLK_DOMAINS_SET_CONTROL: u32 = 0x2080_d01c;

/// Undocumented RM controls for the clock propagation topology, which is what
/// carries the GPC to XBAR ratio. Same caveat as the domain controls above: the
/// layout is private, so it is discovered from the info response rather than
/// assumed, and a write is rejected unless it reads back exactly.
const NV2080_CTRL_CMD_CLK_TOP_PROP_RELS_GET_INFO: u32 = 0x2080_9081;
const NV2080_CTRL_CMD_CLK_TOP_PROP_RELS_GET_CONTROL: u32 = 0x2080_9083;
const NV2080_CTRL_CMD_CLK_TOP_PROP_RELS_SET_CONTROL: u32 = 0x2080_d084;

const CLK_DOMAIN_COUNT: usize = 32;
const CLK_PROP_REL_COUNT: usize = 32;

/// Relationship type that carries a ratio. Other types describe a dependency
/// without one, and pre-Blackwell cards report GPC to XBAR as one of those.
const CLK_PROP_REL_TYPE_RATIO: u8 = 3;

/// Ratios are U16.16 fixed point.
const RATIO_FRACTION_BITS: u32 = 16;

/// The control request starts with this many bytes copied from the info response.
const CLK_PROP_RELS_HEADER_LEN: usize = 0x24;

/// NvAPI clock domain ids of the two ends of the ratio this exposes.
const NV_CLK_DOMAIN_GPC: u32 = 0;
const NV_CLK_DOMAIN_XBAR: u32 = 1;

fn ratio_from_fixed(raw: u32) -> f64 {
    f64::from(raw) / f64::from(1u32 << RATIO_FRACTION_BITS)
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn ratio_to_fixed(ratio: f64) -> u32 {
    (ratio * f64::from(1u32 << RATIO_FRACTION_BITS)).round() as u32
}

/// Index of the MSVDD rail within a domain's per-rail voltage offsets.
const MSVDD_RAIL_INDEX: usize = 1;

/// The only offset mode the driver accepts; the offset is a plain signed kHz value.
const FREQ_OFFSET_MODE_KHZ: u8 = 0;

/// Offsets currently applied to one adjustable clock domain.
#[derive(Debug, Clone, Copy)]
pub struct ClockDomainState {
    /// NvAPI clock domain id, matching [`NvGpuClockDomainId`](super::nvapi::NvGpuClockDomainId).
    pub domain: u32,
    pub freq_offset_khz: i32,
    pub msvdd_offset_uv: i32,
    pub min_offset_mhz: i32,
    pub max_offset_mhz: i32,
}

/// The GPC to XBAR clock propagation ratio.
#[derive(Debug, Clone, Copy)]
pub struct ClockPropagationRatio {
    pub current: f64,
    /// The factory value, which survives writes to the control block.
    pub default: f64,
}

/// Offsets to apply to one clock domain.
#[derive(Debug, Clone, Copy)]
pub struct ClockDomainOffset {
    pub domain: u32,
    pub freq_offset_khz: i32,
    pub msvdd_offset_uv: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct ClkDomainsInfoParams {
    obj_mask: u32,
    /// Bit `i` is set when entry `i` of `domains` is populated.
    domain_mask: u32,
    _reserved: [u8; 0x28],
    domains: [ClkDomainInfo; CLK_DOMAIN_COUNT],
}

/// Board object type of a clock domain, at the start of every info entry.
///
/// The layout of the rest of the entry depends on it, and two of them are known:
/// Blackwell reports `0x10`, while the generations before it report `0x0b` with
/// everything after the parent object shifted four bytes back. An entry of any
/// other type is left alone rather than read at a guessed offset.
///
/// The older layout was measured on an RTX 2000 Ada: with the core clock pinned
/// at 2430MHz by a memory copy, a +300MHz offset on XBAR took it from 2130 to
/// 2430MHz and -300MHz took it to 1830MHz, returning to 2130MHz when cleared.
const CLK_DOMAIN_TYPE_35: u8 = 0x10;
const CLK_DOMAIN_TYPE_3X: u8 = 0x0b;

/// Offset of the frequency offset range within an info entry, per object type.
const RANGE_OFFSET_35: usize = 0x26;
const RANGE_OFFSET_3X: usize = 0x22;

#[repr(C)]
#[derive(Clone, Copy)]
struct ClkDomainInfo {
    obj_type: u8,
    _obj_reserved: [u8; 3],
    /// `1 << domain id`, where the id matches the NvAPI clock domain enum.
    domain_bit: u32,
    /// Read positionally, because the field layout moves with `obj_type`.
    tail: [u8; 0x178],
}

impl ClkDomainInfo {
    /// The allowed frequency offset range in MHz, if the domain accepts offsets.
    ///
    /// Returns `None` for domains that report a zero range, and for object types
    /// whose layout is not known.
    fn offset_range_mhz(&self) -> Option<(i32, i32)> {
        let range_offset = match self.obj_type {
            CLK_DOMAIN_TYPE_35 => RANGE_OFFSET_35,
            CLK_DOMAIN_TYPE_3X => RANGE_OFFSET_3X,
            _ => return None,
        };

        let at = range_offset - mem::offset_of!(Self, tail);
        let min = i16::from_le_bytes(self.tail[at..at + 2].try_into().unwrap());
        let max = i32::from_le_bytes(self.tail[at + 2..at + 6].try_into().unwrap());

        (max > 0).then_some((i32::from(min), max))
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct ClkDomainsControlParams {
    obj_mask: u32,
    domain_mask: u32,
    _reserved: [u8; 0x34],
    domains: [ClkDomainControl; CLK_DOMAIN_COUNT],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct ClkDomainControl {
    obj_type: u32,
    _reserved_1: u32,
    freq_offset_mode: u8,
    _reserved_2: [u8; 3],
    freq_offset_khz: i32,
    /// Per-rail voltage offsets in microvolts. Index [`MSVDD_RAIL_INDEX`] is MSVDD.
    voltage_offsets_uv: [i32; 4],
    _reserved_3: [u8; 0x20],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct ClkPropRelsInfoParams {
    obj_mask: u32,
    /// Bit `i` is set when relationship `i` exists.
    relation_mask: u32,
    _reserved: [u8; 0x120],
    relations: [ClkPropRelInfo; CLK_PROP_REL_COUNT],
    /// Trailing data the driver returns and this does not interpret. It is carried
    /// through writes untouched.
    _tail: [u8; 0x1170],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct ClkPropRelInfo {
    rel_type: u8,
    _reserved_1: u8,
    /// Indices into the clock domain array, not domain ids.
    source_index: u8,
    dest_index: u8,
    bidirectional: u8,
    _reserved_2: [u8; 7],
    /// Factory ratio, U16.16.
    ratio: u32,
    /// The reciprocal of `ratio`, which the driver keeps for the reverse direction.
    inverse_ratio: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct ClkPropRelsControlParams {
    /// The first 0x24 bytes are copied verbatim from the info response.
    header: [u8; CLK_PROP_RELS_HEADER_LEN],
    relations: [ClkPropRelControl; CLK_PROP_REL_COUNT],
    _tail: [u8; 0xa74],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct ClkPropRelControl {
    rel_type: u8,
    _reserved_1: [u8; 7],
    /// U16.16, only meaningful for [`CLK_PROP_REL_TYPE_RATIO`].
    ratio: u32,
}

// The RM validates the parameter size, so a layout mistake would corrupt the
// request rather than fail cleanly.
const _: () = assert!(mem::size_of::<ClkPropRelInfo>() == 0x14);
const _: () = assert!(mem::size_of::<ClkPropRelsInfoParams>() == 0x1518);
const _: () = assert!(mem::size_of::<ClkPropRelControl>() == 0x0c);
const _: () = assert!(mem::size_of::<ClkPropRelsControlParams>() == 0x0c18);
const _: () = assert!(mem::size_of::<ClkDomainInfo>() == 0x180);
const _: () = assert!(mem::size_of::<ClkDomainsInfoParams>() == 0x3030);
const _: () = assert!(mem::size_of::<ClkDomainControl>() == 0x40);
const _: () = assert!(mem::size_of::<ClkDomainsControlParams>() == 0x83c);

#[cfg(feature = "display-info")]
pub fn connector_id_to_display_id(connector_id: u32, drm_device: RawFd) -> anyhow::Result<u32> {
    let mut params = drm_nvidia_get_dpy_id_for_connector_id_params {
        connectorId: connector_id,
        dpyId: 0,
    };
    unsafe {
        get_dpy_id_for_connector_id(drm_device, &raw mut params)?;
    }
    Ok(params.dpyId)
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn alloc_object<T>(
    root: NvU32,
    parent: NvU32,
    class: NvU32,
    alloc_params: Option<&mut T>,
    nvidiactl_fd: RawFd,
) -> anyhow::Result<NvU32> {
    let mut request = NVOS64_PARAMETERS {
        hRoot: root,
        hObjectParent: parent,
        hObjectNew: 0,
        hClass: class,
        pAllocParms: alloc_params.map_or(ptr::null_mut(), |params| ptr::from_mut(params).cast()),
        pRightsRequested: ptr::null_mut(),
        paramsSize: 0,
        flags: 0,
        status: 0,
    };

    rm_alloc_nvos64(nvidiactl_fd, &raw mut request)?;

    if request.status != 0 {
        bail!(
            "Got error status {} on Nvidia object class {class} allocation",
            request.status
        );
    }

    Ok(request.hObjectNew)
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

ioctl_readwrite!(
    get_dpy_id_for_connector_id,
    DRM_IOCTL_BASE,
    DRM_COMMAND_BASE + DRM_NVIDIA_GET_DPY_ID_FOR_CONNECTOR_ID,
    drm_nvidia_get_dpy_id_for_connector_id_params
);
