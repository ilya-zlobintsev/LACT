# Linux AMDGPU Control Application

<img src="res/io.github.lact-linux.png" alt="icon" width="100"/>

This application allows you to control your AMD GPU on a Linux system.

|                                              |                                              |                                             |
|----------------------------------------------|----------------------------------------------|---------------------------------------------|
|![Screenshot](https://i.imgur.com/crEN4az.png)|![Screenshot](https://i.imgur.com/x7fTKpT.png)|![Screenshot](https://i.imgur.com/idAER4B.png)
 

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
- Otherwise, build from source:

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
You can now use the application.

# API
There is an API available over a unix socket. See [here](API.md) for more information.

# CLI

There is also a cli available.

- Getting basic information: 

    `lact cli info`

    Example output:

    ```
    GPU Model: Radeon RX 570 Pulse 4GB
    GPU Vendor: Advanced Micro Devices, Inc. [AMD/ATI]
    Driver in use: amdgpu
    VBIOS Version: 113-1E3871U-O4C
    VRAM Size: 4096
    Link Speed: 8.0 GT/s PCIe
    ```
- Getting current GPU stats:

    `lact cli metrics`

    Example output:

    ```
    VRAM Usage: 545/4096MiB
    Temperature: 46°C
    Fan Speed: 785/3200RPM
    GPU Clock: 783MHz
    GPU Voltage: 0.975V
    VRAM Clock: 1750MHz
    Power Usage: 38/155W
    ```
    
- Showing the current fan curve: 

    `lact cli curve status`
    
    Example output:

    ```
    Fan curve:
    20C°: 0%
    40C°: 0%
    60C°: 50%
    80C°: 88%
    100C°: 100%
    ```

# Reporting issues
 
When reporting issues, please include your system info and GPU model.
 
If there's a crash, run `lact gui` from the command line to get logs, or use `journalctl -u lactd` to see if the daemon crashed.
 

# Alternatives

If LACT doesn't do what you want, make sure to check out [CoreCtrl](https://gitlab.com/corectrl/corectrl).
