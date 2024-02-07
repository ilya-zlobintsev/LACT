# Linux AMDGPU Control Application

<img src="res/io.github.lact-linux.png" alt="icon" width="100"/>

This application allows you to control your AMD GPU on a Linux system.

| GPU info                                     | Overclocking                                 | Fan control                                 |
|----------------------------------------------|----------------------------------------------|---------------------------------------------|
|![image](https://i.imgur.com/gur90cK.png)|![image](https://i.imgur.com/BAL3MgC.png)|![image](https://i.imgur.com/VsAVdOR.png)|

Current features:

- Viewing information about the GPU
- Power/thermals monitoring
- Fan curve control
- Overclocking (GPU/VRAM clockspeed, voltage)
- Power states configuration

# Installation

- Arch Linux: Install the [AUR Package](https://aur.archlinux.org/packages/lact/) (or the -git version)
- Debian/Ubuntu/Derivatives: Download a .deb from [releases](https://github.com/ilya-zlobintsev/LACT/releases/).

  It is only available on Debian 12+ and Ubuntu 22.04+ as older versions don't ship gtk4.
- Fedora: an RPM is available in [releases](https://github.com/ilya-zlobintsev/LACT/releases/).
- Gentoo: Available in [GURU](https://github.com/gentoo/guru/tree/master/sys-apps/lact).
- OpenSUSE: an RPM is available in [releases](https://github.com/ilya-zlobintsev/LACT/releases/).

  Only tumbleweed is supported as leap does not have the required dependencies in the repos.
- NixOS: There is a package available on the [unstable channel](https://search.nixos.org/packages?channel=unstable&from=0&size=50&sort=relevance&type=packages&query=lact)
- Otherwise, build from source.

**Why is there no AppImage/Flatpak/other universal format?**
See [here](./pkg/README.md).

# Usage

Enable and start the service (otherwise you won't be able to change any settings):
```
sudo systemctl enable --now lactd
```
You can now use the GUI to change settings and view information.

**Socket permissions setup:**

By default, LACT uses either ether the `wheel` or `sudo` group (whichever is available) for the ownership of the unix socket that the GUI needs to connect to.

On most configurations (such as the default setup on Arch-based, most Debian-based or Fedora systems) you do not need to do anything.

However, some systems may have different user configuration. In particular, this has been reported to be a problem on OpenSUSE.

To fix socket permissions in such configurations, edit `/etc/lact/config.yaml` and add your username or group as the first entry in `admin_groups` under `daemon`, and restart the service (`sudo systemctl restart lactd`).

# Configuration

There is a configuration file available in `/etc/lact/config.yaml`. Most of the settings are accessible through the GUI, but some of them may be useful to be edited manually (like `admin_groups` to specify who has access to the daemon)

# Overclocking

The overclocking functionality is disabled by default in the driver. There are two ways to enable it:
- By using the "enable overclocking" option in the LACT GUI. This will create a file in `/etc/modprobe.d` that enables the required driver options. This is the easiest way and it should work for most people.

  **Note:** This will attempt to automatically regenerate the initramfs to include the new settings. It does not cover all possible distro combinations. If you've enabled overclocking in LACT but it still doesn't work fter a reboot,
  you might need to check your distro's configuration to make sure the initramfs was updated. Updating the kernel version is a guaranteed way to trigger an initramfs update.
- Specifying a boot parameter. You can manually specify the `amdgpu.ppfeaturemask=0xffffffff` kernel parameter in your bootloader to enable overclocking. See the [ArchWiki](https://wiki.archlinux.org/title/AMDGPU#Boot_parameter) for more details.

# Hardware support
Tested GPU generations:
- [X] Polaris (RX 500 series)
- [X] Vega
- [X] RDNA1 (RX 5000 series)
- [X] RDNA2 (RX 6000 series)
- [X] RDNA3 (RX 7000 series) - Requires Kernel 6.7+

GPUs not listed here will still work, but might not have full functionality available.
Monitoring/system info will be available everywhere. Integrated GPUs might also only have basic configuration available.

# Suspend/Resume

As some of the GPU settings may get reset when suspending the system, LACT will reload them on system resume. This may not work on distributions which don't use systemd, as it relies on the `org.freedesktop.login2` DBus interface.

# Building from source

Dependencies:
- rust
- gtk4
- pkg-config
- make
- hwdata
- libdrm
- blueprint-compiler 0.10.0 or higher (Ubuntu 22.04 in particular ships an older version in the repos, you can manually download a [deb file](http://de.archive.ubuntu.com/ubuntu/pool/universe/b/blueprint-compiler/blueprint-compiler_0.10.0-3_all.deb) of a new version)

Steps:
- `git clone https://github.com/ilya-zlobintsev/LACT && cd LACT`
- `make`
- `sudo make install`

It's also possible to build LACT without some of the features by using cargo feature flags.
This can be useful if some dependency is not available on your system, or is too old.

Build without DRM support (some GPU information will not be available):
```
cargo build --no-default-features -p lact --features=lact-gui
```

Minimal build (no GUI!):
```
cargo build --no-default-features -p lact
```

Build GUI with libadwaita support:
```
make build-release-libadwaita
```

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
