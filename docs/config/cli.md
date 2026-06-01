# CLI Reference

## Options

### General

#### `--config <FILE>`

Path to an explicit configuration file (TOML/YAML/JSON)

#### `--console` (Windows only)

Show console window

- Default: `false`
- Env var: `SISR_CONSOLE`

#### `--api.listen-address <host:port>`

API listen address; by default will pick a random free port and listen on localhost only

- Default: `localhost:0`
- Env var: `SISR_API_LISTEN_ADDRESS`

#### `--api.cors-origins <origins>`

CORS allowed origins

- Default: `https://steamloopback.host,http://steamloopback.host`
- Env var: `SISR_API_CORS_ORIGINS`

#### `--viiper.address <host:port>`, `--va <host:port>`

VIIPER server address. Specify if VIIPER is run manually or on another machine

- Default: `localhost:3242`
- Env var: `SISR_VIIPER_ADDRESS`

#### `--viiper.password <password>`, `--vp <password>`

VIIPER server password. Required for non-localhost connections

Password can be found on the machine running VIIPER in:

- Windows: `%APPDATA%\VIIPER\viiper.key.txt`
- Linux (user): `~/.config/github.com/Alia5/viiper/viiper.key.txt`
- Linux (root/systemd): `/etc/viiper/viiper.key.txt`

- Env var: `SISR_VIIPER_PASSWORD`

#### `--keyboard-mouse-emulation`, `--kbm`

Forward keyboard and mouse when running over a network

- Default: `false`
- Env var: `SISR_KBM_EMULATION`

#### `--update-notify <CHANNEL>`

Update notification level

- Allowed: `none`, `stable`, `prerelease`, `always`
- Default: `stable`
- Env var: `SISR_UPDATE_NOTIFY`

!!! info "Keyboard/Mouse Emulation"

    Can only be used if the VIIPER-server is running on a different machine.  
    If VIIPER address resolves to localhost, this option is ignored

### Controller Emulation

#### `--default-controller-type <TYPE>`

Default controller type for emulation

- Allowed: `xbox360`, `dualshock4`, `dualsense`, `dualsenseedge`, `ns2pro`
- Default: `xbox360`
- Env var: `SISR_DEFAULT_CONTROLLER_TYPE`

#### `--gyro-passthrough [true|false]`, `--gp`

Enable gyro passthrough for supported controllers

- Default: `true`
- Env var: `SISR_GYRO_PASSTHROUGH`

#### `--touchpad-passthrough [true|false]`, `--tp`

Enable touchpad passthrough for supported controllers (e.g. DualShock 4, DualSense)

- Default: `true`
- Env var: `SISR_TOUCHPAD_PASSTHROUGH`

#### `--back-button-passthrough [true|false]`, `--bbp`

Enable back button (paddle/grip button) passthrough for supported controllers (e.g. DualSense Edge, NS2 Pro)

- Default: `false`
- Env var: `SISR_BACK_BUTTON_PASSTHROUGH`

#### `--allow-steam-desktop-layout [true|false]`

Allow/use Steam's desktop configuration for emulated controllers

- Default: `false`
- Env var: `SISR_ALLOW_STEAM_DESKTOP_LAYOUT`

#### `--immediate-sensor-updates [true|false]`

Immediately send sensor updates to VIIPER instead of waiting for the next input report;  
may reduce latency at the cost of increased CPU usage

- Default: `true`
- Env var: `SISR_IMMEDIATE_SENSOR_UPDATES`

#### `--no-steam [true|false]`

Run in no-Steam mode

- Default: `false`
- Env var: `SISR_NO_STEAM`

### Window

#### `--w`, `--window.show [true|false]`

Shows window on startup; when used with fullscreen enabled, this will enable Steam Overlay

- Default: `false`
- Env var: `SISR_SHOW_WINDOW`

#### `--f`, `--window.fullscreen [true|false]`

Create a transparent, borderless, always-on-top overlay window

- Default: `true`
- Env var: `SISR_FULLSCREEN`

#### `--window.max-fps <FPS>`

Maximim FPS for SteamOverlay/UI (Does not affect inputs)

- Default: `60`
- Env var: `SISR_MAX_FPS`

### Logging

#### `--l`, `--log.level <LEVEL>`

Logging level

- Allowed: `trace`, `debug`, `info`, `warn`, `error`
- Default: `info`
- Env var: `SISR_LOG_LEVEL`

#### `--log.file <FILE>`

Write logs to a file

- Default path:
    - Windows: `%APPDATA%/SISR/data/SISR.log`
    - Linux: `~/.config/SISR/data/SISR.log`
- Env var: `SISR_LOG_FILE`

### Steam

#### `--steam.install-dir <PATH>`

Explicit Steam installation path  
_(normally autodetected)_

- Env var: `SISR_STEAM_INSTALL_DIR`

#### `--steam.user-id <ID>`

Active Steam user ID (optional, will attempt to auto-detect if not set)

- Env var: `SISR_STEAM_USER_ID`

#### `--steam.cef-remote-debug-port <PORT>`

CEF remote debugging port (optional, will attempt to auto-detect if not set)

- Env var: `SISR_STEAM_CEF_REMOTE_DEBUG_PORT`

### Special

#### `--marker`

The first found non-Steam game in your Steam library with this argument is used as a "marker shortcut"  
The marker shortcuts Steam Input controller configuration is used when SISR is not launched directly from Steam  
