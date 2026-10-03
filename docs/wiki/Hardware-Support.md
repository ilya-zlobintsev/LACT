## AMD

LACT for the most part does not implement features on a per-generation basis, rather it exposes the functionality that is available in the driver for the current system.
However the following table shows what functionality can be expected for a given generation.

- ✅ - Supported
- ✅* - Supported with certain caveats
- ❔ - Unknown, but theoretically supported
- ❓ - Unknown
- ❌ - Unsupported

| Generation                          | Clocks configuration | Power limit | Power states | Fan control | Notes                                             |
|-------------------------------------|----------------------|-------------|--------------|-------------|---------------------------------------------------|
| RDNA4 (RX 9000)                     | ✅           | ✅    | ❔   | ✅   | Requires recent kernel (6.14+) and firmware versions. Fan zero rpm configuration requires kernel 6.15+. |
| RDNA3 (RX 7000)                     | ✅            | ✅   | ✅    | ✅  | Fan zero RPM mode is enabled by default even with a custom fan curve, and requires kernel 6.13 to be disabled. The power cap is sometimes reported lower than it should be. See [#255](https://github.com/ilya-zlobintsev/LACT/issues/255) for more info.   | 
| RDNA2 (RX 6000)                     | ✅            | ✅   | ✅    | ✅   |                                                   |
| RDNA1 (RX 5000)                     | ✅            | ✅   | ✅    | ✅   |                                                   |
| Vega                                | ✅            | ✅   | ✅    | ✅   |                                                   |
| Arctic Islands/Polaris (RX 400-500) | ✅            | ✅   | ✅    | ✅   | Switching fan control mode back to automatic (including on system shutdown) results in broken fan control and coil whine. This is a known driver-side bug. |
| Volcanic Islands (R7/R9 300)        | ❌            | ✅   | ✅    | ✅   |                                                   |
| Sea Islands (R7/R9 200)             | ❌            | ❓   | ❔    | ❔   | `amdgpu` driver required (see below) |
| Southern Islands (HD 7000)          | ❌            | ❓   | ❓    | ❓   | `amdgpu` driver required (see below) |

Southern Islands and Sea Islands GPUs (also known as GCN1 and GCN2) need the `amdgpu` driver for more functionality to be available.
Starting with kernel 6.19, it's the default driver for them. On older versions you need to explicitly enable it with the following kernel options:
- `amdgpu.si_support=1 radeon.si_support=0` (Southern Islands)
- `amdgpu.cik_support=1 radeon.cik_support=0` (Sea Islands)

GPUs not listed here will still work, but might not have full functionality available.
Monitoring/system info will be available everywhere. Integrated GPUs might also only have basic configuration available.

### Fan control on RDNA3 and newer

Starting with RDNA3, custom fan curves are handled by the GPU's firmware on its own, rather than through LACT setting the fan speed in real time. 

This means that certain features are not supported, such as selecting which sensor is used for the curve. This is a hardware limitation.

## Nvidia

Anything Maxwell (900 series) or newer should generally work for most features.

Locked clocks functionality is supported on Volta+ (16/20 series for consumer cards) for GPU clocks and Ampere+ (30 series) for VRAM clocks.

A recent driver version is highly recommended, as older versions have shown to have issues with certain configuration options such as clockspeed settings. Clockspeed offsets are recommended to be used on driver 565 or newer, and will not show up on versions lower than 555.

## Intel

Functionality status on Intel GPUs:
- Clocks configuration - works on most devices, but there is no support for overclocking (clocks can only be adjusted within the default limits)
- Power limit - works on ARC dGPUs. The maximum power limit might not be reported by the GPU, so the UI might not reflect it appropriately
- Monitoring - most values are shown on devices where they are applicable, dGPU temperature and fan speed reading might need a recent kernel version
- Fan control - not supported by the driver

Additionally, Battlemage GPUs might have problematic Vulkan drivers which cause broken rendering of the GUI.
A workaround for this issue is to force GTK to use OpenGL for rendering with the `GSK_RENDERER=ngl` env variable.

## Laptops

Fan monitoring or control might not be available on some laptops, because the fan is physically not connected through the GPU. This is a matter of hardware design, do not open issues regarding this. On such systems you should use other software for fan control which handles all fans instead of GPU only, such as [CoolerControl](https://gitlab.com/coolercontrol/coolercontrol).