# Flatpak information

LACT is available as a Flatpak package. 
However, due to sandbox restrictions, there are some extra steps needed to achieve full functionality.

LACT uses a system daemon to interact with the GPU and manage configuration.
For access to hardware and system settings the daemon needs to run as root.

When you run the flatpak and the daemon is not detected, you will get a prompt to install it outside of the sandbox.
This will automatically set up and start the system service. It is installed at `/etc/systemd/system/lactd.service`.

If the setup was successful, you will now be able to use all of the functionality.

Service status can be checked with `systemctl status lactd`.

It is also possible to skip the service setup if you want to use LACT only for information and monitoring. This can work entirely in the Flatpak sandbox and does not require extra permissions.

> Note: the setup script requires Polkit to be functional.

## Uninstall

To uninstall the flatpak-created service run the following commands:
```bash
sudo systemctl disable --now lactd
sudo rm /etc/systemd/system/lactd.service
```

## Implementation details

The service uses a binary distributed with the flatpak, but runs it outside of the sandbox.

This is achieved by first getting the location where the LACT flatpak and its flatpak runtime are located on the host.
These paths are used to run the daemon binary, while letting it use libraries provided by the flatpak runtime. This avoids any reliance on libraries being present or missing the host¹.

Additionally, a `flatpak run` wrapper command is provided to the service so that it can call `vulkaninfo` inside of the flatpak sandbox.


1. Nvidia's management library will still be loaded from the host, as it is not directly a part of the Flatpak runtime. The library is shipped with Nvidia drivers, so it is extremely unlikely that it is available in Flatpak while being missing on the host.
