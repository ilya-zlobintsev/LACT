Overclocking functionality (called Overdrive) is disabled by default in the amdgpu driver. It is required to use the following functionality:
- Clocks and voltage configuration
- Power cap configuration beyond the default value
- Fan control (RDNA3+, older generations don't need overdrive for this)

Enabling overdrive requires the `ppfeaturemask` option to be set in the amdgpu driver. There are 2 way to do it:
- For mainstream Linux distributions, you can use the "Enable Overclocking" button in the OC page of the LACT UI. It will automatically create a file with the required setting in `/etc/modprobe.d` and regenerate the initramfs to include it. This has been tested to work with the default setup on Debian, Fedora, and Arch-based systems.
  - This option will not work on distributions that aren't explicitly supported, or if you have a non-standard initramfs configuration. 
  - If this fails, try the boot parameter option.
- Add the setting as a boot parameter. You need to add `amdgpu.ppfeaturemask=0xffffffff` as a kernel parameter in the boot options. The exact procedure for how to do this will depend on your distro.
  - ArchWiki: https://wiki.archlinux.org/title/AMDGPU#Boot_parameter
  - Gentoo wiki: https://wiki.gentoo.org/wiki/AMDGPU#PowerPlay_feature_mask

To disable overdrive:
- If you enabled it through LACT, select the "disable overclocking" option in the dropdown menu.
- If you set the boot option yourself, remove it.

### NixOS

To enable overdrive on NixOS, you can set the following flag in system configuration:
```nix
hardware.amdgpu.overdrive.enable = true;
```

### Fedora Atomic (Silverblue, Kionite, Bazzite etc)

Note: if you're using Bazzite, overdrive is enabled automatically when you run `ujust install-lact`


Enable overdrive:
```
rpm-ostree kargs --append-if-missing=$(printf 'amdgpu.ppfeaturemask=0x%x\n' "$(($(cat /sys/module/amdgpu/parameters/ppfeaturemask) | 0x4000))")
```


Disable overdrive:
```
rpm-ostree kargs --delete-if-present=$(printf 'amdgpu.ppfeaturemask=0x%x\n' "$(($(cat /sys/module/amdgpu/parameters/ppfeaturemask) | 0x4000))")
```