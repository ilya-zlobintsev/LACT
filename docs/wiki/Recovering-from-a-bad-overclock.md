If you've overclocked your GPU into an unstable state, you might want to reset the overclock settings to defaults.

Whenever you change OC related settings, LACT will wait for the settings to be confirmed in a dialog before saving them to the system config.
In most cases this prevents you from saving settings that cause a GPU crash, letting you reset them through the UI normally.

![image](https://github.com/user-attachments/assets/3f432632-b458-4e3a-a453-57a374c1841b)


However, in some situations it is possible that overclock settings have already been confirmed and saved to config, but found to be unstable later. In the worst-case scenario, the unstable overclock might cause a GPU crash before being able to log into a graphical session and reset the settings.

In order to recover from this situation LACT supports a special boot option. If `lact-reset` is specified in the kernel boot parameters, the config will get reset and the old one will be backed up under a different filename in `/etc/lact`.

The way to specify a boot parameter depends on your distribution, but in systems using GRUB for boot such as Ubuntu and Fedora you can specify parameters by pressing `e` in the boot menu and editing the line that starts with `linux`:

![image](https://github.com/user-attachments/assets/8c376287-d4df-4a9b-99f2-556dad5ec759)

**Note:** you should only specify the `lact-reset` option temporarily to recover your system, keeping it around will cause the LACT config to be reset every time the service starts.