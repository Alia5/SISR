# FAQ

## Where's the GUI?

SISR runs as a **system tray application** by default.

- Right-click the tray icon to show/hide the UI
- Or launch with `-w --window-fullscreen false` to show the window at startup
- **If** the window runs **as overlay** press **`Ctrl+Shift+Alt+S`**
  or **`LB+RB+BACK+A`** (_A button needs to be pressed last_) to toggle UI visibility.

You can also run `sisr --help` to see all CLI options.

## What is USBIP?

**USBIP** is a protocol for tunneling USB devices over TCP/IP  
It allows a USB device on one machine to appear on another machine over the network (or localhost).

SISR uses USBIP (via [VIIPER](https://alia5.github.io/VIIPER/)) to create emulated controllers
that appear as real hardware at the system level

See [USBIP setup](../getting-started/usbip.md) for setup instructions

## What is VIIPER?

VIIPER (**V**irtual **I**nput over **IP** **E**mulato**R**) is the USBIP server that SISR uses to emulate controllers.

**VIIPER is bundled with SISR**  
you don't need to download/setup it separately  

VIIPER listens on:

- `:3241` for USBIP connections
- `:3242` for the control API

See the [VIIPER documentation](https://alia5.github.io/VIIPER/) for more details

## Common Issues

For common issues (doubled controllers, Steam CEF debugging, port conflicts, etc.), see: [Troubleshooting](troubleshooting.md)

## I want feature XYZ

Check [GitHub Issues](https://github.com/Alia5/SISR/issues) to see if it's already requested  
If not, open a new issue  

No guarantees, though.  

Better yet, implement it yourself and open a pull request 😉  
Alternatively, you can hire me to implement it for you 😜  
Rates start at 100€/hour.
