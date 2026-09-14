use crate::server::gpu_controller::PciSlotInfo;
use anyhow::{bail, ensure};

// Public NV01_ROOT queries from NVIDIA's ctrl0000gpu.h. These queries resolve
// PCI identity to the RM device/subdevice instance numbers used by NV0080 and
// NV2080 allocations; neither number is a Linux device minor.
const GET_ATTACHED_IDS: u32 = 0x201;
const GET_ID_INFO_V2: u32 = 0x205;
const GET_PCI_INFO: u32 = 0x21b;
const MAX_GPUS: usize = 32;
const INVALID_GPU_ID: u32 = u32::MAX;

pub(super) fn resolve_gpu_instance(
    pci: &PciSlotInfo,
    mut query: impl FnMut(u32, &mut [u8]) -> anyhow::Result<()>,
) -> anyhow::Result<(u32, u32)> {
    // This RM query exposes domain/bus/slot, but no PCI function. Do not match
    // another function of a multifunction device by silently dropping it.
    ensure!(pci.func == 0, "RM GPU lookup requires PCI function zero");
    let mut attached = [0xff; MAX_GPUS * 4];
    query(GET_ATTACHED_IDS, &mut attached)?;

    for id in attached.chunks_exact(4) {
        let gpu_id = u32::from_ne_bytes(id.try_into()?);
        if gpu_id == INVALID_GPU_ID {
            continue;
        }

        // NV0000_CTRL_GPU_GET_PCI_INFO_PARAMS: u32 gpuId/domain, u16 bus/slot.
        let mut location = [0; 12];
        location[..4].copy_from_slice(id);
        query(GET_PCI_INFO, &mut location)?;
        let domain = u32::from_ne_bytes(location[4..8].try_into()?);
        let bus = u16::from_ne_bytes(location[8..10].try_into()?);
        let slot = u16::from_ne_bytes(location[10..12].try_into()?);
        if (domain, bus, slot) != (u32::from(pci.domain), pci.bus, pci.dev) {
            continue;
        }

        // NV0000_CTRL_GPU_GET_ID_INFO_V2_PARAMS: eight u32 fields, with
        // deviceInstance/subDeviceInstance at +8/+12.
        let mut info = [0; 32];
        info[..4].copy_from_slice(id);
        query(GET_ID_INFO_V2, &mut info)?;
        let device = u32::from_ne_bytes(info[8..12].try_into()?);
        let subdevice = u32::from_ne_bytes(info[12..16].try_into()?);
        return Ok((device, subdevice));
    }

    bail!("No Nvidia RM GPU matches PCI location {pci:?}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_pci_identity_when_minor_and_rm_orders_differ() {
        // This host has Ada at minor 5/RM 4 and the 5090 at minor 4/RM 5.
        // IDs are opaque and enumeration order must not select the device.
        let pci = PciSlotInfo {
            domain: 0,
            bus: 0x0d,
            dev: 0,
            func: 0,
        };
        let instances = resolve_gpu_instance(&pci, |cmd, data| {
            match cmd {
                GET_ATTACHED_IDS => {
                    data[..4].copy_from_slice(&0x2e00_u32.to_ne_bytes());
                    data[4..8].copy_from_slice(&0x0d00_u32.to_ne_bytes());
                }
                GET_PCI_INFO => {
                    let id = u32::from_ne_bytes(data[..4].try_into()?);
                    data[8..10].copy_from_slice(
                        &(if id == 0x2e00 { 0x2e_u16 } else { 0x0d }).to_ne_bytes(),
                    );
                }
                GET_ID_INFO_V2 => {
                    assert_eq!(u32::from_ne_bytes(data[..4].try_into()?), 0x0d00);
                    data[8..12].copy_from_slice(&5_u32.to_ne_bytes());
                    data[12..16].copy_from_slice(&2_u32.to_ne_bytes());
                }
                _ => panic!("unexpected command {cmd:#x}"),
            }
            Ok(())
        })
        .unwrap();
        assert_eq!(instances, (5, 2));
    }

    #[test]
    fn does_not_fall_back_to_another_gpu_when_pci_is_missing() {
        let pci = PciSlotInfo {
            domain: 1,
            bus: 0x0d,
            dev: 0,
            func: 0,
        };
        let result = resolve_gpu_instance(&pci, |cmd, data| {
            match cmd {
                GET_ATTACHED_IDS => data[..4].copy_from_slice(&0x0d00_u32.to_ne_bytes()),
                GET_PCI_INFO => data[8..10].copy_from_slice(&0x0d_u16.to_ne_bytes()),
                _ => panic!("must not allocate a GPU from another PCI domain"),
            }
            Ok(())
        });
        assert!(result.is_err());
    }

    #[test]
    fn propagates_rm_query_failure() {
        let pci = PciSlotInfo {
            domain: 0,
            bus: 0x0d,
            dev: 0,
            func: 0,
        };
        let error = resolve_gpu_instance(&pci, |_, _| bail!("RM unavailable")).unwrap_err();
        assert!(error.to_string().contains("RM unavailable"));
    }
}
