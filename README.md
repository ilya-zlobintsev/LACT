# Linux AMDGPU Control Application

This application allows you to control your AMD GPU on a Linux system.

|                                              |                                              |                                             |
|----------------------------------------------|----------------------------------------------|---------------------------------------------|
|![Screenshot](https://i.imgur.com/MdLYQTz.png)|![Screenshot](https://i.imgur.com/frnZskA.png)|![Screenshot](https://i.imgur.com/TM2VyUb.png)
 

Current features:

- Viewing information about the GPU
- Power/thermals monitoring
- Fan curve control
- Basic overclocking

Currently missing:
- Clockspeed/voltages cannot be changed on Vega20 and newer GPUs due to them using a different power level management logic
- <s>Multi-GPU system support</s> *Should work now*
- The card model detection isn't very reliable

# Installation

- Arch Linux: Install the [AUR Package](https://aur.archlinux.org/packages/lact-git/)
- Anything else:
    - Install a rust toolchain and gtk3 development headers (libgtk-3-dev on ubuntu)
    - Clone the repo
    - ./deploy.sh

# Usage

Enable and start the service (otherwise you won't be able to change any settings):
```
sudo systemctl enable --now lactd
```
You can now use the application.
