# <svg xmlns="http://www.w3.org/2000/svg" style="transform: translate(0px, 10px)" width="1.4em" height="1.4em" viewBox="0 0 48 48"><rect width="15.5" height="15.5" x="5.5" y="5.5" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" rx="2" ry="2" stroke-width="1"/><path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" d="m32.451 6.465l-6.532 11.742C25.223 19.46 26.13 21 27.561 21h13.065c1.433-.002 2.337-1.541 1.64-2.793l-6.53-11.742a1.88 1.88 0 0 0-3.284 0" stroke-width="1"/><circle cx="34.093" cy="34.093" r="8.407" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1"/><path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" d="m6.232 30.613l3.483 3.482l-3.483 3.483a2.5 2.5 0 0 0 3.536 3.535l3.482-3.482l3.482 3.482a2.5 2.5 0 0 0 3.536-3.535l-3.483-3.483l3.483-3.482a2.5 2.5 0 0 0-3.536-3.535L13.25 30.56l-3.482-3.482a2.5 2.5 0 0 0-3.536 3.535" stroke-width="1"/></svg> Playstation Controller Emulation

SISR supports emulating Playstation controllers instead of Xbox 360 controllers.

This can be useful for games and applications with native Playstation controller support, or when Steam Input poses issues or inconveniences.

The following controller types are available:

| Type | Config value |
|------|-------------|
| DualShock 4 | `dualshock4` |
| DualSense | `dualsense` |
| DualSense Edge | `dualsenseedge` |

!!! info "Gyro Passthrough"

    Gyro is automatically passed through **if** the source controller has gyro support.  
    There are a few "gotchas" to be aware of, though:

    - **Gyro calibration:**  
        Normally controllers provide their own gyro calibration data; SISR does not translate this.  
        You will need to calibrate the gyro on the emulated controller itself, either via Steam or in-game options.

    - **Steam Controller (1/2) / Deck specific:**  
        Gyro data is not transmitted from Steam to SISR unless gyro is bound to something **other than "_None_"** in the Steam Input configuration.  
        As a workaround: bind gyro to any non-gyro action (e.g. directional swipe) and leave the actions empty.

    Gyro passthrough is **enabled by default** ( `gyro-passthrough=true`).

!!! info "Touchpad Passthrough"

    Touchpad input (from a Steam Deck, source Playstation controller, or similar) is passed through to the emulated controller's touchpad.

    Applies to: **DualShock 4**, **DualSense**, **DualSense Edge**

    Touchpad passthrough is **enabled by default** (`touchpad-passthrough=true`).

!!! info "Back Button Passthrough"

    Back button (paddle) input from the source controller is passed through to the emulated controller's back buttons.

    Applies to: **DualSense Edge** (back paddles/buttons)

    Back button passthrough is **disabled by default** ( `back-button-passthrough=false`).

## Enabling Playstation Controller Emulation

Pass the desired controller type as a launch argument:

```
SISR.exe --ct=dualshock4
SISR.exe --ct=dualsense
SISR.exe --ct=dualsenseedge
```

For permanent configuration see [Configuration](../config/config.md).
