# Linux AMDGPU Control Application

<img src="res/io.github.lact-linux.png" alt="icon" width="100"/>

This application allows you to control your AMD GPU on a Linux system.

| GPU info                                     | Overclocking                                 | Fan control                                 |
|----------------------------------------------|----------------------------------------------|---------------------------------------------|
|![image](https://user-images.githubusercontent.com/22796665/221402316-7ff140ee-9013-4599-9263-bdbf15906896.png)|![image](https://user-images.githubusercontent.com/22796665/221402327-f5f69c29-8f07-4e3e-9b77-6f064b26d6f0.png)|![image](https://user-images.githubusercontent.com/22796665/221402354-06c1a2a1-4849-4953-99ea-cab94e8413af.png)|

Current features:

- Viewing information about the GPU
- Power/thermals monitoring
- Fan curve control
- Basic overclocking

Currently missing:
- Precise clock/voltage curve manipulation (currently can only set the maximum values)

# Installation

- Arch Linux: Install the [AUR Package](https://aur.archlinux.org/packages/lact/) (or the -git version)
- Debian/Ubuntu/Derevatives: Download a .deb from [releases](https://github.com/ilya-zlobintsev/LACT/releases/).

  It is only available on Debian 12+ and Ubuntu 22.04+ as older versions don't ship gtk4.
- Fedora: an rpm is available in [releases](https://github.com/ilya-zlobintsev/LACT/releases/).
- Otherwise, build from source.

**Why is there no AppImage/Flatpak/other universal format?**
See [here](./pkg/README.md).

# Configuration

There is a configuration file available in `/etc/lact/config.yaml`. Most of the settings are accessible through the GUI, but some of them may be useful to be edited manually (like `admin_groups` to specify who has access to the daemon)

# Building from source

Dependencies:
- rust
- gtk4
- pkg-config
- make
- hwdata

Steps:
- `git clone https://github.com/ilya-zlobintsev/LACT && cd LACT`
- `make`
- `sudo make install`

# Usage

Enable and start the service (otherwise you won't be able to change any settings):
```
sudo systemctl enable --now lactd
```
You can now use the GUI to change settings and view information.

# API
There is an API available over a unix socket. See [here](API.md) for more information.

# CLI

There is also a cli available.

- List system GPUs: 

    `lact cli list-gpus`

    Example output:

    ```
    1002:687F-1043:0555-0000:0b:00.0 (Vega 10 XL/XT [Radeon RX Vega 56/64])
    ```
- Getting GPU information:

    `lact cli info`

    Example output:

    ```
    lact cli info
    GPU Vendor: Advanced Micro Devices, Inc. [AMD/ATI]
    GPU Model: Vega 10 XL/XT [Radeon RX Vega 56/64]
    Driver in use: amdgpu
    VBIOS version: 115-D050PIL-100
    Link: LinkInfo { current_width: Some("16"), current_speed: Some("8.0 GT/s PCIe"), max_width: Some("16"), max_speed: Some("8.0 GT/s PCIe") }
    ```
    
The functionality of the CLI is quite limited. If you want to integrate LACT with some application/script, you should use the [API](API.md) instead.

# Reporting issues
 
When reporting issues, please include your system info and GPU model.
 
If there's a crash, run `lact gui` from the command line to get logs, or use `journalctl -u lactd` to see if the daemon crashed.
 

# Alternatives

If LACT doesn't do what you want, make sure to check out [CoreCtrl](https://gitlab.com/corectrl/corectrl).
