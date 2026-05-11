# <svg xmlns="http://www.w3.org/2000/svg" style="transform: translate(0px, 10px)" width="1.4em" height="1.4em" viewBox="0 0 48 48"><rect width="15.5" height="15.5" x="5.5" y="5.5" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" rx="2" ry="2" stroke-width="1"/><path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" d="m32.451 6.465l-6.532 11.742C25.223 19.46 26.13 21 27.561 21h13.065c1.433-.002 2.337-1.541 1.64-2.793l-6.53-11.742a1.88 1.88 0 0 0-3.284 0" stroke-width="1"/><circle cx="34.093" cy="34.093" r="8.407" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1"/><path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" d="m6.232 30.613l3.483 3.482l-3.483 3.483a2.5 2.5 0 0 0 3.536 3.535l3.482-3.482l3.482 3.482a2.5 2.5 0 0 0 3.536-3.535l-3.483-3.483l3.483-3.482a2.5 2.5 0 0 0-3.536-3.535L13.25 30.56l-3.482-3.482a2.5 2.5 0 0 0-3.536 3.535" stroke-width="1"/></svg> DS4 Emulation

!!! danger "Experimental Feature"

    DualShock4 emulation is an experimental and work in progress feature and may not work as expected in all scenarios.  
    Please open a Discussion on GitHub for any questions or issues you may encounter.

SISR includes experimental support to emulate DualShock4 controllers instead of Xbox360 controllers.  

This can be useful for games/applications with native Playstation controller support or may
otherwise pose issues or inconveniences using Steam Input.

!!! info "Gyro Passthrough"

    Gyro is automatically passed through **if** the source controller has gyro support.  
    There are a few "gotchas" to be aware of, though:  

    - Gyro calibration:  
        Normally controllers provide their own gyro calibration data, SISR does not translate this in any way
        This means that for the emulated DS4 controller you have to calibrate the gyro on the emulated controller itself, be it via Steam or in-game options

    - Steam Controller / Deck specific:  
        Gyro data is not transmitted to SISR unless gyro is bound to something other than "_None_" in Steam Input configuration.  
        As a workaround: Bind gyro to any non-gyro action (e.g. right analog stick) and set the sensitivity as low as possible (or 0%)

!!! warning "Touchpad passthrough"

    Touchpad input (either from the Steam Deck or real Playstation controllers) is currently **not** passed through to the emulated DualShock4 controller!  
    This may be added in a future update.

## Enabling DS4 Emulation

Enabling DS4 emulation in SISR is done via CLI-argument / configuration file

    Pass `--default-controller-type=dualshock4` as launch argument to SISR.  
    For permanent configuration see [Configuration](../../../config/config)

