# <svg xmlns="http://www.w3.org/2000/svg" style="transform: translate(0px, 10px)"  width="1.4em" height="1.4em" viewBox="0 0 1024 1024"><path d="M0 0h1024v1024H0z" fill="none" /><path fill="currentColor" d="M247.5 358.4a72.7 72.7 0 0 0 72.3 72.4c39.9 0 72.4-32.6 72.4-72.4S359.6 286 319.8 286s-72.3 32.6-72.3 72.4" /><path fill="currentColor" d="M492.4 128H324.7a197 197 0 0 0-139 57.6a197 197 0 0 0-57.7 139v374.1a197 197 0 0 0 121.4 181.8a197 197 0 0 0 75.3 15h167.7c3 0 5.4-2.5 5.4-5.5V133.4c.6-3-1.8-5.4-5.4-5.4m-56.1 705.9H324.7a132 132 0 0 1-95.4-39.9a134 134 0 0 1-39.8-95.3v-374c-.1-17.8 3.4-35.4 10.2-51.7a132 132 0 0 1 29.6-43.7a135 135 0 0 1 95.4-39.8h111.6zm402-647.7a197 197 0 0 0-139-57.6H580.5c-3 0-4.8 2.4-4.8 4.8v757.2c-.6 3 1.7 5.4 5.4 5.4h118.2a197 197 0 0 0 139-57.6a197 197 0 0 0 57.7-139V325.2a197 197 0 0 0-57.6-139M727 628c-42.8 0-77.8-35-77.8-77.8s35-77.8 77.8-77.8s77.8 35 77.8 77.8s-35 77.8-77.8 77.8" /></svg> Switch 2 Pro Controller Emulation

SISR supports emulating a Nintendo Switch 2 Pro controller instead of an Xbox 360 controller.

This can be useful for games and applications that have native Switch 2 Pro controller support.

| Type | Config value |
|------|-------------|
| Switch 2 Pro | `ns2pro` |

!!! info "Gyro Passthrough"

    Gyro is automatically passed through **if** the source controller has gyro support.  
    There are a few "gotchas" to be aware of, though:

    - **Gyro calibration:**  
        Normally controllers provide their own gyro calibration data; SISR does not translate this.  
        You will need to calibrate the gyro on the emulated controller itself, either via Steam or in-game options.

    - **Steam Controller (1/2) / Deck specific:**  
        Gyro data is not transmitted from Steam to SISR unless gyro is bound to something **other than "_None_"** in the Steam Input configuration.  
        As a workaround: bind gyro to any non-gyro action (e.g. directional swipe) and leave the actions empty.

    Gyro passthrough is **enabled by default** (`--gyro-passthrough=true`).

!!! info "Back Button Passthrough"

    Back button (grip/paddle) input from the source controller is passed through to the emulated controller's back buttons.

    Back button passthrough is **disabled by default** (`--back-button-passthrough=false`).

## Enabling Switch 2 Pro Emulation

Pass the controller type as a launch argument:

```
SISR.exe --ct=ns2pro
```

For permanent configuration see [Configuration](../config/config.md).
