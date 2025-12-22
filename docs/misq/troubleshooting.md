# Troubleshooting

## 🎮 Controller Issues

### Doubled controllers / One physical controller controls multiple emulated controllers

You can try one of the two following things:

1. Ensure that in the Steam Controller configurator for SISR,
the controller order uses your "real" controllers **before any emulated controllers**  

2. Turn off "Enable Steam Input for Xbox controllers" in Steam settings.  
Otherwise Steam will pass through the emulated controller to SISR, which will then create another virtual  
controller, which will be passed to Steam, which will it pass to SISR, which will then create another virtual  
controller, which will be passed to Steam, which will it pass to SISR, which will then create another virtual
controller, which will be passed to Steam, which will it pass to SISR, which will then create another virtual
controller, which will be passed to Steam, which will it pass to SISR, which will then create another virtual
controller, which will be passed to Steam, which will it pass to SISR, which will then create another virtual
controller, which will be passed to Steam, which will it pass to SISR, which will then create another virtual
controller, which will be passed to Steam, which will it pass to SISR, which will then create another virtual
controller, which will be passed to Steam, which will it pass to SISR, which will then create another virtual
controller, which will be passed to Steam, which will it pass to SISR, which will then create another virtual
controller, which will be passed to Steam, which will it pass to SISR.

!!! info "Controller identification"
    Steams "Identify Controllers" feature (available when re-ordering controller **in Steam**) will
    help you differentiate physical and emulated controllers

### My game still detects my real PS4/DualSense/Nintendo controller

Install and use [HidHide](https://github.com/nefarius/HidHide) to hide your physical controllers from games  
Keep the visible to Steam and SISR  
_How?_ **RTFM**...

!!! info "HidHide setup"
    Automatic HidHide integration will maybe follow  
    soon™

### Game doesn't recognize the controller

Does the game work with regular, real, Xbox 360 controllers?  

- If yes, you are doing it wrong  
- If no, tough luck

### Touch/Radial menus do not work

By default, SISR will **not** draw continuously, which prevents touch/radial menus from showing up  
To make touch/radial menus show up, you can add the `--wcd true` launch option to enable continuous drawing  

This can also circumvent issues with the Steam overlay not showing up correctly.

**Do note that this may increase CPU/GPU usage** and
can potentially negatively affect gaming performance on lower end systems

## 🪟 UI / Window issues

### I can't see the UI / The UI doesn't show up

- It's a system tray app. Right-click the tray icon to toggle the UI (among other things)
- Or launch with `-w --window-fullscreen false` to show the window at startup
- **If** the window runs **as overlay** press **`Ctrl+Shift+Alt+S`**
  or **`LB+RB+BACK+A`** (_A button needs to be pressed last_) to toggle UI visibility.

### I have toggled the UI but now I can't get rid of it

- Press **`Ctrl+Shift+Alt+S`** or **`LB+RB+BACK+A`** (_"A" button needs to be pressed last_) again to toggle UI visibility

### My mouse is captured by the overlay and I can't interact with other windows

- Press **`Ctrl+Shift+Alt+S`** or **`LB+RB+BACK+A`** (_"A" button needs to be pressed last_) to toggle UI visibility

## 🐍 VIIPER Issues

### SISR says VIIPER is unavailable

1. Is VIIPER running?  
    Start manually: `viiper server`
2. Is `viiper` / `viiper.exe` next to SISR?
    SISR tries to auto-start it if not already running as a service and the viiper-address is set to `localhost`
3. Firewall blocking the connection?  
    Allow VIIPER through your firewall
4. Correct address?  
    Default is `localhost:3242`. Change with `--viiper-address`
5. **If** using remote VIIPER: Is the remote machine reachable?  
    Try pinging it

### VIIPER version too old

SISR enforces a minimum VIIPER version  
VIIPER should come bundled with SISR, so this should not happen

If you see this error, you likely use VIIPER on another machine or have VIIPER running as a service
In any case check the [VIIPER Documentation](https://alia5.github.io/VIIPER/) for update instructions

### USBIP attach fails

Ensure you have USBIP set up correctly  
See [USBIP setup](../getting-started/usbip.md)

## 🚂 Steam Integration

### SISR marker not found

SISR reports the marker shortcut is missing.

Create it manually:

1. Add SISR as a **non-Steam Game** in Steam
2. Set launch options to `--marker`
3. Restart Steam and SISR

See [Installation](../getting-started/installation.md)

### Port 8080 conflicts / CEF debugging is enabled, but SISR could not reach it

As do other popular tools, SISR uses the CEF-Debugging option provided by Steam  
and Valve decided to default that to port 8080 (_without an easy way to change this permanently_)

Stop the conflicting service/program ¯\\\_(ツ)\_/¯  

### Steam installation could not be found

Ensure Steam is installed and the installation directory exists  
On Windows, check the registry entry for Steam  

You can also specify the path explicitly with `--steam-path`

### Failed to create CEF debug enable file in Steam directory

SISR couldn't write to the Steam directory (permissions issue, antivirus, etc.)

Manually create the file `.cef-enable-remote-debugging` in your Steam installation directory  
See [Installation](../getting-started/installation.md)

### Failed to restart Steam

SISR couldn't restart Steam automatically via `steam://` URL scheme
Restart Steam manually, then restart SISR

## ⌨️🖱️ Keyboard/Mouse Emulation

### KB/M emulation is disabled

SISR disables KB/M emulation on **localhost/loopback** as it makes no sense there  

To enable: Run VIIPER on a different machine and run SISR with `--viiper-address=<remote-ip>:3242 --keyboard-mouse-emulation=true`

## 🏎️ Performance

### Input lag

Check:

- Network latency (if using remote VIIPER): ping the host
- System performance: CPU/GPU usage, background processes
- Game settings: V-sync, frame rate limits

!!! info
    USBIP/VIIPER do **not** introduce significant latency  
    See [VIIPER benchmarks](https://alia5.github.io/VIIPER/main/testing/e2e_latency/)

## Still stuck? 🙄

Open an issue on [GitHub](https://github.com/Alia5/SISR/issues) with:

- SISR version
- OS and version
- VIIPER version
- Relevant log output (`--log-level=debug`)
- Steps to reproduce

No guarantees of support, though.  
