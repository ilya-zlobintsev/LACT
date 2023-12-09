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
- Fedora: an rpm is available in [releases](https://github.com/ilya-zlobintsev/LACT/releases/).
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

# Configuration

There is a configuration file available in `/etc/lact/config.yaml`. Most of the settings are accessible through the GUI, but some of them may be useful to be edited manually (like `admin_groups` to specify who has access to the daemon)

# Overclocking

The overclocking functionality is disabled by default in the driver. There are two ways to enable it:
- By using the "enable overclocking" option in the LACT GUI. This will create a file in `/etc/modprobe.d` that enables the required driver options. This is the easiest way and it should work for most people.

  **Note:** you might need to regenerate the initramfs for the setting to be applied. On Arch-based systems, this means running `mkinitcpio -P`.
- Specifying a boot parameter. You can manually specify the `amdgpu.ppfeaturemask=0xffffffff` kernel parameter in your bootloader to enable overclocking. See the [ArchWiki](https://wiki.archlinux.org/title/AMDGPU#Boot_parameter) for more details.

# Hardware support
Tested GPU generations:
- [X] Polaris (RX 500 series)
- [X] Vega
- [X] RDNA1 (RX 5000 series)
- [X] RDNA2 (RX 6000 series)
- [ ] RDNA3 (RX 7000 series) - overclocking is not available on stable kernel versions, and is [expected to land in Linux 6.7](https://gitlab.freedesktop.org/drm/amd/-/issues/2840#note_2079945)

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
 
If there's a crash, run `lact gui` from the command line to get logs, or use `journalctl -u lactd` to see if the daemon crashed.
 

# Alternatives

If LACT doesn't do what you want, make sure to check out [CoreCtrl](https://gitlab.com/corectrl/corectrl).
