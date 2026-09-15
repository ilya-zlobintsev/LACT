# NVIDIA power limits below the VBIOS minimum

LACT can request a power limit below the VBIOS minimum when the NVIDIA driver
exposes a compatible ordinary power-policy client. Discovery uses read-only
RM queries and compares their results with NVML. It does not require a specific
driver version string and does not change power settings during discovery.

The extended minimum is 30 W, or the native minimum if that is already lower.
The VBIOS maximum remains the upper bound. Values within the native range
and resets to the default continue to use NVML.

## Compatibility detection

Two native wire formats are recognized. LACT tries their information GETs,
then accepts a format only when:

- The response has the expected header and only the first client in its mask.
- Minimum, default and maximum values are valid and agree with NVML in mW.
- The ordinary FE client's current request agrees with NVML.
- The client selector is unchanged and its request is neither zero nor absent.

The selected format is retained for writes. LACT rereads bounds and control
before applying a change, modifies only the request field, and checks the full
readback. On SET or readback failure it restores the previous payload using
that same format, including when the previous request was below the native
minimum. A restoration failure is reported. No write is used to try alternative
formats, and the additional F8 client is not modified.

Unknown or inconsistent formats retain the native NVML range. This is support
for compatible interfaces, not a guarantee that every past or future NVIDIA
driver accepts lower power limits. A driver may reject the SET even after its
GET layout is recognized; LACT reports the failure rather than saving success.

## Versions compared

The three command sizes were independently checked in each version's NvAPI
translation library and both its GA10x and TU10x GSP dispatch tables:

| Driver | Information GET a630 | Control GET a632 / SET e633 | Native request / selector offsets |
|---|---|---|---|
| 595.45.04 | 0x488 | 0x188 | 0x0c / 0x10 |
| 595.58.03 | 0x488 | 0x188 | 0x0c / 0x10 |
| 595.71.05 | 0x488 | 0x188 | 0x0c / 0x10 |
| 610.43.02 | 0x924 | 0x328 | 0x2c / 0x30 |
| 610.43.03 | 0x924 | 0x328 | 0x2c / 0x30 |
| 610.57.04 | 0x924 | 0x328 | 0x2c / 0x30 |
| 615.71.09 | 0x924 | 0x328 | 0x2c / 0x30 |

Command prefix: `0x2080`. These are private native payloads, excluding NvAPI's
transport prefix. Information min/default/max offsets are 0x0c/0x10/0x14 in
the smaller format and 0x28/0x2c/0x30 in the larger format. The local source
folder named `5955809` actually identifies itself as 595.58.03. Two local
610.57.04 source trees with different P2P changes use the same driver version.

These comparisons establish layout compatibility. **Live below-minimum
requests have been validated on 610.57.04; the other versions were inspected
offline, not installed for a GPU trial.** Future versions using either known
format can pass the same runtime checks without adding their version number
to LACT. A different format requires explicit support.

## Expected behavior and upgrades

Requested watts use NVIDIA's readings. LACT does not compensate for a shunt
modification. A board reported as consuming 150 W can physically consume more
after such a modification; any conversion factor must come from the hardware.

A low request does not guarantee the same actual consumption. In the original
610.57.04 PRO 6000 trial, 100 W produced approximately 100 W of reported
consumption, while a 30 W request still consumed approximately 74 W after ten
seconds with the mining workload and memory clocks unchanged.

After a driver upgrade, LACT repeats discovery. A compatible format can retain
the extended range even if the version number changes. If discovery fails,
a saved request below the native minimum is rejected during application; it
is not silently clamped. Check the active limit after loading the new driver.

Implementation: [power_limit.rs](../lact-daemon/src/server/gpu_controller/nvidia/driver/power_limit.rs).
Unit tests cover both formats, GET-only discovery, malformed headers/masks,
NVML mismatches, request isolation, bounds, restoration and restoration errors.
