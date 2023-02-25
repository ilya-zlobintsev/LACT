# What is this

This is a directory used by [pkger](https://github.com/vv9k/pkger/) to generate packages for different distros from a single manifest.

Usage:
```
pkger -c .pkger.yml build lact
```
(Should be ran in the repo root).

Generated packages will be placed in `pkg/output`.

# Why is there no AppImage/Flatpak/Docker?

Unfortunately, due to the nature of the app that has 2 parts (daemon running as root and a GUI running as a user), none of the popular universal formats fit it very well.

---
Flatpak is mainly designed for graphical desktop applications, with support for user CLI apps as well.

It does not support services, and running a Flatpak as root is also problematic. This means that the LACT daemon can't run as Flatpak.

---
AppImage on the other hand can run as root. However it is also mainly designed for graphical apps where you download a run a single file. 

This means that you would need to manually install a service file alongside the AppImage, though this could be automated with a script. 

The bigger problem is that AppImages are built by taking the dependencies of the host system. Some things (primarly `glibc`) are usually not bundled with the image, which means that the recommended way to package  things is by building the AppImage on the oldest possible system that you want your app to run on. This is not really possible with LACT, as it uses gtk4, which means that the "universal" AppImage would only run on modern distros anyway.

It is possible to bundle glibc with the AppImage by using [appimage-builder](https://appimage-builder.readthedocs.io/), which would enable the image to run on older systems. The problem is that the resulting file ends up being very large, and since AppImage extract itself on startup it means that it takes about 10 seconds to start every time even on a relatively powerful system. There are flags that allow AppImages to reuse files between restarts, but it ends up storing hundreds of megabytes of data on /tmp (which is stored in RAM on most systems). Overall it is not a very good user experience.

---
The daemon could run in a Docker container with little issue. However running graphical applications in Docker, while possible, is extremely inconvenient and doesn't integrate with the rest of the system.

---
All of this means that native packaging is currently the only feasible way to distribute LACT.