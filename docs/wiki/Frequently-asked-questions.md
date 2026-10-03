- ## "Could not connect to daemon, running in embedded mode"
  
  This message appears when the LACT GUI cannot connect to the system service. It usually happens because either:
  - The service is not running
  - Your user does not have permissions to access it

  Firs of all, make sure the daemon is enabled and running. You can see the status of the daemon in `systemctl status lactd`.
  If the daemon is running, and you're getting a "permission denied" error, you need to set permissions for who can access the daemon by setting either `admin_user` or `admin_group` in the config file: see [here](https://github.com/ilya-zlobintsev/LACT?tab=readme-ov-file#configuration) for how to do it.

- ## LACT Flatpak is using light theme despite system settings
  
  LACT on its own respects the system theme, but Flatpak does not have integration for GTK4 theming: see [this issue](https://github.com/ilya-zlobintsev/LACT/issues/585#issuecomment-3068875175) for explanation links and workarounds.

  You can also set the `GTK_THEME=Adwaita:dark` env variable using a tool such as Flatseal to force the a specific theme.

- ## Fan control not working on a laptop

  Fan monitoring or control might not be available on some laptops, because the fan is physically not connected through the GPU. This is a matter of hardware design, as LACT specifically queries the GPU for fans, and the system just doesn't have one. On such setups you should use other software for fan control which handles all system fans, such as [CoolerControl](https://gitlab.com/coolercontrol/coolercontrol).

- ## How to enable overclocking on AMD

  See [here](https://github.com/ilya-zlobintsev/LACT/wiki/Overclocking-(AMD))

- ## How to undervolt Nvidia GPUs

    **The below text is outdated, as there is a VF curve editor now. It is kept here since the old method might still be useful.**

    Nvidia GPUs don't expose voltage control directly, but it is possible to achieve a pseudo-undervolt by combining the locked clocks option with a positive clockspeed offset.
    This will force the GPU to run at a voltage that's constrained by the locked clocks, while achieving a higher clockspeed due to the offset.

  See [#486](https://github.com/ilya-zlobintsev/LACT/issues/486) for more details and some example setups.

