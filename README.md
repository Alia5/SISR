<img src="SISR.svg" align="right" width="128"/>
<br />

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

> ⚠️ **Highly experimental work in progress.** Everything is subject to change and may or may not work.  
Expect bugs, crashes, and missing features.

## Still here? 🤔 Okay here's how to get it running

1. Make sure you have USBIP setup on your system  
    - On Windows, install [USBIP-Win2](https://github.com/OSSign/vadimgrn--usbip-win2/releases)  
    (use the latest **Pre**-Release)
    - On Linux, install the `usbip` package (or whatever package includes this on whatever you are running) and load the `vhci-hcd` kernel modules
2. Start Steam
3. Start a [VIIPER](https://github.com/Alia5/VIIPER) server on your system  
   Use the latest **Pre**-Release, and use the CLI.
4. Start SISR. ¯\\\_(ツ)_/¯  
    - If the automatic first time setup does not work (probable, as it's just a series of warnings and error dialogs ;P):
      1. Create an empty file names `.cef-enable-remote-debugging` in **your steam installation directory** (e.g. `C:\Program Files (x86)\Steam` / `~/.steam/steam`)
      2. Add SISR as a non-Steam game in your Steam library  
       Set the launch options of that shortcut to `--marker`
      3. Restart Steam, then restart SISR

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
  I do this in my spare time; it's better to have something that provides value than whatever elitist kind of mental masturbation.

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

