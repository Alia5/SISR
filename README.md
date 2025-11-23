[![Build Status](https://github.com/alia5/SISR/actions/workflows/snapshots.yml/badge.svg)](https://github.com/alia5/SISR/actions/workflows/snapshots.yml)
[![License: GPL-3.0](https://img.shields.io/github/license/alia5/SISR)](https://github.com/alia5/SISR/blob/main/LICENSE.txt)
[![Release](https://img.shields.io/github/v/release/alia5/SISR?include_prereleases&sort=semver)](https://github.com/alia5/SISR/releases)
[![Issues](https://img.shields.io/github/issues/alia5/SISR)](https://github.com/alia5/SISR/issues)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](https://github.com/alia5/SISR/pulls)
[![Downloads](https://img.shields.io/github/downloads/alia5/SISR/total?logo=github)](https://github.com/alia5/SISR/releases)

# SISR ✂️

**S**team **I**nput **S**ystem **R**edirector

SISR (pronounced "scissor") is a tool that allows users to redirect Steam Input configurations to a system level, either on localhost or even over the network.

Unlike it's predecessor [GlosSI](https://github.com/Alia5/GlosSI), SISR uses [VIIPER](https://github.com/Alia5/VIIPER) _(requiring **USBIP**)_ instead of the unmaintained [ViGEm](https://github.com/ViGEm/ViGEmBus) driver, to emulate virtual controllers.  

> ⚠️ **Highly experimental work in progress.** Everything is subject to change and may or may not work. Expect bugs, crashes, and missing features.

## ⚙️ How It Works

1. Add SISR as a non-Steam game to your Steam library
2. Add the following launch arguments to SISR's Steam entry  
   `--window.create=true --window.fullscreen=false --console`
3. Launch a VIIPER server on your system _(will be bundled in future releases, **maybe, soon™**)_
4. Launch SISR through Steam (so Steam Input can process your controllers)
5. SISR captures Steam-processed gamepad inputs and creates virtual Xbox 360 controllers via VIIPER
6. Launch your games normally (not through Steam) - they'll see the virtual controllers
7. Configure your controllers using Steam's Input Configurator while SISR is running

## 📦 Dependencies

- **[VIIPER](https://github.com/Alia5/VIIPER)** server must be running on your system
- **SISR must be added as a non-Steam game** and launched through Steam

## 😭 Mimimi (FAQ)

### "Mimimi, I get doubled controllers"

- Turn off "Enable Steam Input for Xbox controllers" in Steam settings.  
Otherwise Steam will pass through the emulated controller to SISR, which will then create another emulated controller, resulting in duplicates.

### "Mimimi, the game still detects my _real_ PS4/DualSense/whatever controller"

- Setup [HidHide](https://github.com/nefarius/HidHide) to hide your physical controllers from games, **RTFM**.  
Automatic HidHide integration will (maybe) follow whenever soon™.

### "Mimimi, it doesn't work with my game"

- Does the game work with regular Xbox 360 controllers? If yes, file an issue with logs. If no, tough luck.

### "Mimimi, where's the GUI?"

- It's a system tray app. Right-click the tray icon for options. What more do you want?  
  You could also run `./sisr --help` ¯\\\_(ツ)\_/¯

### "Mimimi, I want feature XYZ back 😭"

- Code it yourself and open up a PR.  
  Alternatively, hire me to do it for you - Rates start at 100€/hour.

### "Mimimi, your code is shit / you're doing it wrong"

- Cool story bro. Where's your pull request?

## 📝 TODO

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
