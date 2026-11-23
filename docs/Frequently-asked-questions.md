- ## "Could not connect to daemon, running in embedded mode"
  
  This message appears when the LACT GUI cannot connect to the system service. It usually happens because either:
  - The service is not running
  - Your user does not have permissions to access it

  First of all, make sure the daemon is enabled and running. You can see the status of the daemon in `systemctl status lactd`.
  If the daemon is running, and you're getting a "permission denied" error, you need to set permissions for who can access the daemon by setting either `admin_user` or `admin_group` in the config file: see [here](https://github.com/ilya-zlobintsev/LACT?tab=readme-ov-file#configuration) for how to do it.

- ## LACT Flatpak is using a light theme despite system settings
  
  LACT on its own respects the system theme, but Flatpak does not have integration for GTK4 theming: see [this issue](https://github.com/ilya-zlobintsev/LACT/issues/585#issuecomment-3068875175) for explanation links and workarounds.

  You can also set the `GTK_THEME=Adwaita:dark` env variable using a tool such as Flatseal to force a specific theme.

- ## Fan control not working on a laptop

  Fan monitoring or control might not be available on some laptops, because the fan is physically not connected through the GPU. This is a matter of hardware design, as LACT specifically queries the GPU for fans, and the system just doesn't have one. On such setups you should use other software for fan control which handles all system fans, such as [CoolerControl](https://gitlab.com/coolercontrol/coolercontrol).

- ## How to enable overclocking on AMD

  See [here](./Overclocking-(AMD).md)
