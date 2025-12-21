# CLI Reference

## Options

### General

#### `-c`, `--config <FILE>`

Path to an explicit configuration file (TOML/YAML/JSON)

#### `--cli`, `--console` (Windows only)

show console window

- Default: `false`

#### `-t`, `--tray [true|false]`

Enable the system tray icon

- Default: `true`
- Env var: `SISR_TRAY`

#### `--viiper-address <host:port>`, `--va <host:port>`

VIIPER API-server address. Specify if VIIPER is run manually or on another machine

- Default: `localhost:3242`
- Env var: `SISR_VIIPER_ADDRESS`

#### `--keyboard-mouse-emulation [true|false]`, `--kbm [true|false]`

Emulate/forward keyboard and mouse inputs

- Default: `false`
- Env var: `SISR_KBM_EMULATION`

!!! info "Keyboard/Mouse Emulation"

    Can only be used if the VIIPER-server is running on a different machine.  
    If VIIPER-address resolves to localhost, this option is ignored

### Window

#### `-w`, `--window-create [true|false]`

Create/show window at launch

- Default: `false`
- Env var: `SISR_WINDOW_CREATE`

#### `-f`, `--window-fullscreen [true|false]`

Create a transparent, borderless, always-on-top overlay window  
Window is fully transparent and click-through

- Default: `true`
- Env var: `SISR_WINDOW_FULLSCREEN`

#### `--window-continous-draw [true|false]`, `--wcd [true|false]`

Continuously update/redraw the window  
Use when Steam overlay detection fails or other overlay issues occur  
May increase CPU/GPU usage

- Default: `false`
- Env var: `SISR_WINDOW_CONTINOUS_DRAW`

### Logging

#### `-l`, `--log-level <LEVEL>`

Logging level

- Allowed: `error`, `warn`, `info`, `debug`, `trace`
- Default: `info`
- Env var: `SISR_LOG_LEVEL`

#### `--log-file <FILE> [FILE_LEVEL]`

Write logs to a file

- Default path:
    - Windows: `%APPDATA%/SISR/data/SISR.log`
    - Linux: `~/.config/SISR/data/SISR.log`
- Env var: `SISR_LOG_FILE`

File logging level used together with `--log-file`

- Allowed: `error`, `warn`, `info`, `debug`, `trace`
- Default: same as `--log-level`

### Steam

#### `--disable-steam-cef-debug [true|false]`

Disable Steam CEF remote debugging

- Default: `false`
- Env var: `SISR_STEAM_CEF_DEBUG_DISABLE`

#### `--steam-launch-timeout-secs <SECONDS>`

Time to wait for Steam to launch

- Default: `1`
- Env var: `SISR_STEAM_LAUNCH_TIMEOUT_SECS`

#### `--steam-path <PATH>`

Explicit Steam installation path  
_(normally autodetected)_

- Env var: `SISR_STEAM_PATH`

### Special

#### `--marker`

The first found non-Steam game in your Steam library with this argument is used as a "marker shortcut"  
The marker shortcuts Steam Input controller configuration is used when SISR is not launched directly from Steam  
