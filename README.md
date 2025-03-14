# Linux GPU Control Application

<img src="res/io.github.lact-linux.png" alt="icon" width="100"/>

This application allows you to control your AMD, Nvidia or Intel GPU on a Linux system.

| GPU info                                     | Overclocking                                 | Fan control                                 |
|----------------------------------------------|----------------------------------------------|---------------------------------------------|
|![image](https://i.imgur.com/gur90cK.png)|![image](https://i.imgur.com/BAL3MgC.png)|![image](https://i.imgur.com/VsAVdOR.png)|
| Historical data |
|![image](https://i.imgur.com/GOmKh1M.png)|

Current features:

- Viewing information about the GPU
- Power and thermals monitoring, power limit configuration
- Fan curve control (AMD and Nvidia)
- Overclocking (GPU/VRAM clockspeed and voltage)
- Power states configuration (AMD only)

All of the functionality works regardless of the desktop session (there is no dependency on X11 extensions).

# Installation

- Arch Linux: Install the package from official repositories: `pacman -S lact` (or `lact-git` from AUR for development builds).
- Debian/Ubuntu/Derivatives: Download a .deb from [releases](https://github.com/ilya-zlobintsev/LACT/releases/).

  It is only available on Debian 12+ and Ubuntu 22.04+ as older versions don't ship gtk4.
- Fedora: use the [Copr repository](https://copr.fedorainfracloud.org/coprs/ilyaz/LACT/), or download an RPM from [releases](https://github.com/ilya-zlobintsev/LACT/releases/).
- Gentoo: Available in [GURU](https://github.com/gentoo/guru/tree/master/sys-apps/lact).
- OpenSUSE: an RPM is available in [releases](https://github.com/ilya-zlobintsev/LACT/releases/).

  Only tumbleweed is supported as leap does not have the required dependencies in the repos.
- NixOS: There is a package available in [nixpkgs](https://search.nixos.org/packages?channel=24.05&from=0&size=50&sort=relevance&type=packages&query=lact)
- Otherwise, build from source.

**Why is there no AppImage/Flatpak/other universal format?**
See [here](./pkg/README.md#why-is-there-no-appimageflatpakdocker).

Note: Nvidia support requires the Nvidia proprietary driver with CUDA libraries installed.

## Development builds

To get latest fixes or features that have not yet been released in a stable version, there are packages built from the latest commit that you can install from the [test release](https://github.com/ilya-zlobintsev/LACT/releases/tag/test-build) or using the `lact-git` AUR package on Arch-based distros.

Note: the date of the test release is not the date when the packages were built, the actual date is specified next to the attached package files.

# Usage

Enable and start the service (otherwise you won't be able to change any settings):
```
sudo systemctl enable --now lactd
```
You can now use the GUI to change settings and view information.

# Hardware support

See the [Wiki page](https://github.com/ilya-zlobintsev/LACT/wiki/Hardware-Support)

# Configuration

There is a configuration file available in `/etc/lact/config.yaml`. Most of the settings are accessible through the GUI, but some of them may be useful to be edited manually (like `admin_groups` to specify who has access to the daemon)

See [CONFIG.md](./docs/CONFIG.md) for more information.

**Socket permissions setup:**

By default, LACT uses either ether the `wheel` or `sudo` group (whichever is available) for the ownership of the unix socket that the GUI needs to connect to.

On most configurations (such as the default setup on Arch-based, most Debian-based or Fedora systems) you do not need to do anything.

However, some systems may have different user configuration. In particular, this has been reported to be a problem on OpenSUSE.

To fix socket permissions in such configurations, edit `/etc/lact/config.yaml` and add your username or group as the first entry in `admin_groups` under `daemon`, and restart the service (`sudo systemctl restart lactd`).

# Overclocking (AMD)

Some functionality requires enabling an option in the amdgpu driver, see the [wiki page](https://github.com/ilya-zlobintsev/LACT/wiki/Overclocking-(AMD)) for more information.

## Power profiles daemon note!

If you are using `power-profiles-daemon` (which is installed by default on many distributions), by default it will override the amdgpu performance level setting according to its own profile.

To avoid this, create a file at `/etc/systemd/system/power-profiles-daemon.service.d/override.conf` with the following contents:
```
[Service]
ExecStart=
ExecStart=/usr/libexec/power-profiles-daemon --block-action=amdgpu_dpm
```
See https://github.com/ilya-zlobintsev/LACT/issues/370 for more information.

# Suspend/Resume

As some of the GPU settings may get reset when suspending the system, LACT will reload them on system resume. This may not work on distributions which don't use systemd, as it relies on the `org.freedesktop.login2` DBus interface.

# Building from source

Dependencies:
- rust 1.76+
- gtk 4.6+
- git
- pkg-config
- clang
- make
- hwdata
- libdrm

Command to install all dependencies:
- Fedora: `sudo dnf install rust cargo make git clang gtk4-devel libdrm-devel vulkan-tools`
- Arch: `sudo pacman -S --needed base-devel git clang make rust gtk4 hwdata vulkan-tools`

Steps:
- `git clone https://github.com/ilya-zlobintsev/LACT && cd LACT`
- `make`
- `sudo make install`

It's possible to change which features LACT gets built with. 
To do so, replace the `make` command with the following variation:

Headless build with no GUI:
```
make build-release-headless
```

Build GUI with libadwaita support:
```
make build-release-libadwaita
```

# API

There is an API available over a unix or TCP socket. See [here](docs/API.md) for more information.

# Remote management

It's possible to have the LACT daemon running on one machine, and then manage it remotely from another.

This is disabled by default, as the TCP connection **does not have any authentication or encryption mechanism!**
Make sure to only use it in trusted networks and/or set up appropriate firewall rules.

To enable it, edit `/etc/lact/config.yaml` and add `tcp_listen_address` with your desired address and in the `daemon` section.

Example:
```yaml
daemon:
  tcp_listen_address: 0.0.0.0:12853
  log_level: info
  admin_groups:
  - wheel
  - sudo
  disable_clocks_cleanup: false
```

After this restart the service (`sudo systemctl restart lactd`).

To connect to a remote instance with the GUI, run it with `lact gui --tcp-address 192.168.1.10:12853`.

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

If you're having an issue with changing the GPU's configuration, it's highly recommended to include a debug snapshot in the bug report.
You can generate one using the option in the dropdown menu:

![image](https://github.com/ilya-zlobintsev/LACT/assets/22796665/36dda5e3-981b-47e7-914e-6e29f30616b4)

The snapshot is an archive which includes the SysFS that LACT uses to interact with the GPU.
 
If there's a crash, run `lact gui` from the command line to get GUI logs, check daemon logs in `journalctl -u lactd` for errors, 
and see `dmesg` for kernel logs that might include information about driver and system issues.

# Other tools

Here's a list of other useful tools for AMD GPUs on Linux:
- [CoreCtrl](https://gitlab.com/corectrl/corectrl) - direct alternative to LACT, provides similar functionality in addition to CPU configuration with a Qt UI
- [amdgpu_top](https://github.com/Umio-Yasuno/amdgpu_top) - tool for detailed real-time statistics on AMD GPUs
- [Tuxclocker](https://github.com/Lurkki14/tuxclocker) - Qt overclocking tool, has support for AMD GPUs
