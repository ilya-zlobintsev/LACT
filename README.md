# Linux AMDGPU Control Application

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
- Voltage control on Vega20+ GPUs
- Precise clock/voltage curve manipulation (currently can only set the maximum values)
- <s>Multi-GPU system support</s> *Should work now*

# Installation

- Arch Linux: Install the [AUR Package](https://aur.archlinux.org/packages/lact-git/)
- Anything else:
    - Install dependencies:
      - Ubuntu/Debian: `sudo apt install cargo rustc libvulkan-dev git libgtk-3-dev make`
      - Fedora: `sudo dnf install git gtk3-devel rust cargo vulkan-headers perl-FindBin perl-File-Compare`

    - `git clone https://github.com/ilyazzz/LACT && cd LACT`
    - `./deploy.sh` 


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

# Reporting issues
 
 When reporting issues, please include your system info and GPU model.
 
 If there's a crash, run `lact-gui` from the command line to get logs, or use `journalctl -u lactd` to see if the daemon crashed.
 
 If there's an issue with GPU model identification please report it [here](https://github.com/ilyazzz/pci-id-parser/), include your GPU model and the output of `cat /sys/class/drm/card*/device/uevent`.
