# Hardware support

## AMD

LACT for the most part does not implement features on a per-generation basis, rather it exposes the functionality that is available in the driver for the current system.
However the following table shows what functionality can be expected for a given generation.

- **Supported** - the functionality is known to work
- **Limited** - the functionality is known to work, but has certain limitations
- **Untested** - the functionality has not been confirmed to work, but it should
- **Unknown** - the functionality has not been confirmed to work, and it is unknown if it does
- **Unsupported** - the functionality is known to not work

| Generation                          | Clocks configuration | Power limit | Power states | Fan control | Notes                                             |
|-------------------------------------|----------------------|-------------|--------------|-------------|---------------------------------------------------|
| Southern Islands (HD 7000)          | Unsupported          | Unknown     | Unknown      | Untested    | Requires the `amdgpu.si_support=1` kernel option  |
| Sea Islands (R7/R9 200)             | Unsupported          | Unknown     | Untested     | Untested    | Requires the `amdgpu.cik_support=1` kernel option |
| Volcanic Islands (R7/R9 300)        | Unsupported          | Unknown     | Untested     | Untested    |                                                   |
| Arctic Islands/Polaris (RX 400-500) | Supported            | Supported   | Supported    | Supported   |                                                   |
| Vega                                | Supported            | Supported   | Supported    | Supported   |                                                   |
| RDNA1 (RX 5000)                     | Supported            | Supported   | Supported    | Supported   |                                                   |
| RDNA2 (RX 6000)                     | Supported            | Supported   | Supported    | Supported   |                                                   |
| RDNA3 (RX 7000)                     | Supported            | Supported   | Supported    | Supported   | Fan zero RPM mode is enabled by default even with a custom fan curve, and requires kernel 6.13 to be disabled. The power cap is sometimes reported lower than it should be. See [#255](https://github.com/ilya-zlobintsev/LACT/issues/255) for more info.   | 

GPUs not listed here will still work, but might not have full functionality available.
Monitoring/system info will be available everywhere. Integrated GPUs might also only have basic configuration available.

## Nvidia

Anything Maxwell (900 series) or newer should generally work. 
A recent driver version is highly recommended, as older versions have shown to have issues with certain configuration options such as clockspeed settings.

## Intel

Functionality status on Intel GPUs:
- Clocks configuration - works on most devices, but there is no support for overclocking (clocks can only be adjusted within the default limits)
- Power limit - works on ARC dGPUs. The maximum power limit might not be reported by the GPU, so the UI will change depending on the current limit
- Monitoring - most values are shown on devices where they are applicable, dGPU temperature and fan speed reading might need a recent kernel version
- Fan control - not supported by the driver
