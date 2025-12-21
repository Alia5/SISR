<img src="sisr.svg" align="right" width="128"/>
<br />

# SISR ✂️

**S**team **I**nput **S**ystem **R**edirector

SISR (pronounced "scissor") redirects Steam Input configurations to the system level (localhost or network).  

It can be used to circumvent issues with games and applications that
do not support Steam Input or otherwise pose challenges, like (but not limited to):

- Games with aggressive anti-cheat systems
- Emulators
- Windows Store games/apps
- Games with broken Steam Input support

SISR can also be used to "tunnel"/forward Steam Input configurations over the network to other machines, including Keyboard/Mouse.  
This makes it possible to use devices like a Steam Deck as a dedicated controller without the need to stream the entire game.

The emulated controllers (and Keyboard/Mouse) are indistinguishable from real hardware and show up at system level.  
SISR achieves this by utilizing [VIIPER](https://github.com/Alia5/VIIPER) (requires **USBIP**).  
Unlike its predecessor [GlosSI](https://github.com/Alia5/GlosSI), it does not use the unmaintained [ViGEm](https://github.com/ViGEm/ViGEmBus) driver.

!!! warning
    Highly experimental WIP. Expect bugs, crashes, and missing features.

## ✨🛣️ Features / Roadmap

- ✅ Steam Input redirection to system level (localhost or network)  
    - Indistinguishable from real hardware
- ✅ Xbox 360 controller emulation
- ✅ Keyboard/Mouse emulation (only in network scenarios)  
    - Allows use of devices like the Steam Deck as dedicated controller
- ✅ Flexible configuration (CLI, config files, environment variables)
- ✅ Multi-platform support (Windows, Linux)
- ✅ Multiple operation modes
    - Standalone background service
    - Steam overlay window mode
- 🚧 PS4 controller emulation
- 🚧 Xbox One controller emulation
- 🚧 Generic controller emulation
- 🚧 Gyro Passthrough
- 🚧 Bundling multiple devices into a single controller
- 🚧 Automatic HidHide integration

## 🚀 Getting started

- [Installation](getting-started/installation.md)

## ⚙️ Configuration

- [Configuration](config/config.md)
- [CLI Reference](config/cli.md)

## 🆘 Help

- [Troubleshooting](misq/troubleshooting.md)
- [FAQ](misq/faq.md)

## 🛠️ Development

- [Building](dev/building.md)

## 🔗 Links

- [📥 Downloads](downloads/index.md)
- [GitHub Repository](https://github.com/Alia5/SISR)
- [SISR Releases](https://github.com/Alia5/SISR/releases)
- [VIIPER Docs](https://alia5.github.io/VIIPER/)
- [USBIP-Win2 (Windows USBIP)](https://github.com/vadimgrn/usbip-win2)
- [Changelog](changelog/)
