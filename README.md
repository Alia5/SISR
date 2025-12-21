<img src="docs/SISR.svg" align="right" width="128"/>
<br />

[![Build Status](https://github.com/alia5/SISR/actions/workflows/snapshots.yml/badge.svg)](https://github.com/alia5/SISR/actions/workflows/snapshots.yml)
[![License: GPL-3.0](https://img.shields.io/github/license/alia5/SISR)](https://github.com/alia5/SISR/blob/main/LICENSE.txt)
[![Release](https://img.shields.io/github/v/release/alia5/SISR?include_prereleases&sort=semver)](https://github.com/alia5/SISR/releases)
[![Issues](https://img.shields.io/github/issues/alia5/SISR)](https://github.com/alia5/SISR/issues)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](https://github.com/alia5/SISR/pulls)
[![Downloads](https://img.shields.io/github/downloads/alia5/SISR/total?logo=github)](https://github.com/alia5/SISR/releases)

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

> ⚠️ **Highly experimental work in progress.** Everything is subject to change and may or may not work.  
Expect bugs, crashes, and missing features.

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

## How to get it running

Read the [documentation](https://alia5.github.io/SISR/)!

## 😭 Mimimi (FAQ)

### "Mimimi, I get doubled controllers" / "Mimimi only one of my controllers controls multiple emulated controllers"

You can try one of the two following things:

1. Ensure that in the Steam Controller configurator for SISR, the controller order uses your "real" controllers **before any emulated controllers**.

2. Turn off "Enable Steam Input for Xbox controllers" in Steam settings.  
Otherwise Steam will pass through the emulated controller to SISR, which will then create another virtual controller, which will be passed to Steam, which will it pass to SISR, which will then create another virtual controller, which will be passed to Steam, which will it pass to SISR, which will then create another virtual controller, which will be passed to Steam, which will it pass to SISR, which will then create another virtual controller, which will be passed to Steam, which will it pass to SISR, which will then create another virtual controller, which will be passed to Steam, which will it pass to SISR, which will then create another virtual controller, which will be passed to Steam, which will it pass to SISR, which will then create another virtual controller, which will be passed to Steam, which will it pass to SISR, which will then create another virtual controller, which will be passed to Steam, which will it pass to SISR, which will then create another virtual controller, which will be passed to Steam, which will it pass to SISR, which will then create another virtual controller, which will be passed to Steam, which will it pass to SISR.

### "Mimimi, the game still detects my _real_ PS4/DualSense/whatever controller"

- Setup [HidHide](https://github.com/nefarius/HidHide) to hide your physical controllers from games, **RTFM**.  
Automatic HidHide integration will (maybe) follow whenever soon™.

### "Mimimi, it doesn't work with my game"

- Does the game work with regular Xbox 360 controllers?  
  If yes, you are doing it wrong.  
  If no, tough luck.

### "Mimimi, where's the GUI?"

- It's a system tray app. Right-click the tray icon to show a window?  
  You could also run `./sisr --help` to see what options are available.  
  What more do you want? ¯\\\_(ツ)\_/¯

### "Mimimi, touch menus do not work"

- Not implemented.

### "Mimimi, I can only have one Steam Input config active"

- **Nope.**  
   Just add SISR multiple times as non-Steam game (this time **without** `--marker` launch option) and launch that ;)

### "Mimimi port 8080 is blocked/used"

- Thank Valve for that.  
  As do other popular tools, SISR uses the CEF-Debugging option provided by Steam, and Valve decided to default to port 8080, not easily changeable via a config-file.

### "Mimimi, USBIP is slow, mimimi VIIPER also uses TCP mimimi. This causes input lag"

- **Nope.**  
  If you are experiencing input lag, it's another issue.  
  See the E2E benchmarks from VIIPER.

### "Mimimi, I want feature XYZ 😭"

- Code it yourself and open up a PR.  
  Alternatively, hire me to do it for you - Rates start at 100€/hour.

### "Mimimi, your code is shit / you're doing it wrong"

- Cool story bro. Where's your pull request?  
  I do this in my spare time; it's better to have something that provides value than whatever elitist kind of mental masturbation you're after.

## 📝 Contributing

PRs welcome! See [GitHub Issues](https://github.com/Alia5/SISR/issues) for open tasks.

## 📄 License

```license
SISR - Steam Input System Redirector

Copyright (C) 2025 Peter Repukat

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
```
