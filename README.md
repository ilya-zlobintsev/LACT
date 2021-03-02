# Linux AMDGPU Control Application

This application allows you to control your AMD GPU on a Linux system.

|                                              |                                              |                                             |
|----------------------------------------------|----------------------------------------------|---------------------------------------------|
|![Screenshot](https://i.imgur.com/AqwkWKT.png)|![Screenshot](https://i.imgur.com/3VpQ0vC.png)|![Screenshot](https://i.imgur.com/okW7aq2.png)
 

Current features:

- Viewing information about the GPU
- Power/thermals monitoring
- Fan curve control
- Basic overclocking

Currently missing:
- Voltage control on Vega20+ GPUs
- Precise clock/voltage curve manipulation (currently can only set the maximum values)
- <s>Multi-GPU system support</s> *Should work now*
- The card model detection isn't very reliable

# Installation

- Arch Linux: Install the [AUR Package](https://aur.archlinux.org/packages/lact-git/)
- Fedora:
    - `sudo dnf install git gtk3-devel rust cargo vulkan-headers`
    - `git clone https://github.com/ilyazzz/LACT && cd LACT`
    - `./deploy.sh`
- Ubuntu/Debian:
    - `sudo apt install cargo rustc libvulkan-dev git libgtk3-dev`
    - `git clone https://github.com/ilyazzz/LACT && cd LACT`
    - `./deploy.sh`
- Anything else:
    - Install a rust toolchain, gtk3 dev headers and vulkan header
    - Clone the repo
    - ./deploy.sh

# Usage

Enable and start the service (otherwise you won't be able to change any settings):
```
sudo systemctl enable --now lactd
```
You can now use the application.

# CLI

There is also a cli available.

- Getting basic information: 

    `lact-cli info`

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

    `lact-cli metrics`

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

    `lact-cli curve status`
    
    Example output:

    ```
    Fan curve:
    20C°: 0%
    40C°: 0%
    60C°: 50%
    80C°: 88%
    100C°: 100%
    ```