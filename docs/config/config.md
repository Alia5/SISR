# Configuration

SISR can be configured via:

1. CLI flags  
  See [CLI Reference](cli.md)
1. Environment variables
1. Config files (TOML/YAML/JSON)

!!! info "Configuration merging"
    SISR merges defaults + any discovered config files + your explicit `--config` file + CLI overrides  
    (_in that order_)

## Common options

<style>
#config-options th, #config-options td { padding: 1em 0.5em; place-content: center; }
#config-options code { white-space: nowrap; word-break: keep-all; }
#config-options td:last-child { white-space: nowrap; }
</style>

<table style="width:100%; border-collapse: collapse;" border="1" id="config-options">
  <thead>
    <tr>
      <th>Option</th>
      <th>Description</th>
      <th>Default</th>
    </tr>

  </thead>
  <tbody>
    <tr>
      <td>
        <div><code>--c file</code></div>
        <div><code>--config file</code></div>

      </td>
      <td>specify config file</td>
      <td></td>
    </tr>

    <tr>
      <td>
        <div><code>-w</code></div>
        <div><code>--window-create [true|false]</code></div>
      </td>
      <td>Create/Show window at launch</td>
      <td><code>false</code></td>
    </tr>

    <tr>
      <td>
        <div><code>-f</code></div>
        <div><code>--window-fullscreen  [true|false]</code></div>
      </td>
      <td>
        Create a transparent, borderless,always on top,
          overlay window.<br />
          used as an overlay-target<br />
          Window is fully transparent and click-through
      </td>
      <td><code>true</code></td>
    </tr>

    <tr>
      <td>
        <div><code>-wcd</code></div>
        <div><code>--window-continous-draw  [true|false]</code></div>
      </td>
      <td>

          Continously update/draw to the window.<br />
          May be used when steam oerlay detection fails
          or other issues with the steam overlay occur

      </td>
      <td><code>true</code></td>
    </tr>

    <tr>
      <td><code>--viiper-address &lt;hostname:port&gt;</code></td>
      <td>
          VIIPER API-server address<br />
          Can be specified if VIIPER is run manually,
          or on another machine
      </td>
      <td><code>localhost:3242</code></td>
    </tr>

    <tr>
      <td><code>--default-controller-type &lt;type&gt;</code></td>
      <td>
          Set the default controller type that should be emulated<br />
          Possible values: "xbox360", "dualshock4"
      </td>
      <td><code>"xbox360"</code></td>
    </tr>

    <tr>
      <td>
        <div><code>--kbm</code></div>
        <div><code>--keyboard-mouse-emulation</code></div>
      </td>
      <td>
          Emulate/Forward Keyboard and Mouse inputs<br />
          Can only be used if the VIIPER-server
          is running on a different machine
      </td>
      <td><code>false</code></td>
    </tr>
  </tbody>
</table>

## Config file discovery

SISR looks for these names and extensions:

- Names: `SISR`, `config`
- Extensions: `.toml`, `.yaml`, `.yml`, `.json`

Search locations:

- Your platform config directory
    - Windows: `%APPDATA%\SISR\config` (example: `C:\Users\<UserName>\AppData\Roaming\SISR\config`)
    - Linux: `$XDG_CONFIG_HOME/sisr` (or `~/.config/sisr`)
- The directory next to the SISR executable

You can also explicitly provide a config file path via `--config <FILE>`.

### Discovery order and precedence

When multiple sources provide the same configuration option, the latest one in the following is picked:

1. Defaults
2. Discovered config files (in this exact order):  
     1. In the platform config dir:
     2. Next to the SISR executable:
3. Your explicit `--config <FILE>` (if provided)
4. CLI flags and environment variables

## Full example (default) configuration

All values are optional.

🪟 `C:\Users\<UserName>\AppData\Roaming\SISR\config\SISR.toml`  
🐧 `$XDG_CONFIG_HOME/sisr/SISR.toml`

```toml
# Windows only: show console window
# On other platforms this key is ignored
console = false

# Enable/disable system tray icon
tray = true

# VIIPER API server address
viiper_address = "localhost:3242"

# Enable keyboard/mouse emulation.
# Will only work if the specified VIIPER server does not run on localhost
kbm_emulation = false

[controller_emulation]
# Default controller type for emulation
# Allowed: "xbox360", "dualshock4"
default_controller_type = "xbox360"

require_controllers_connected_before_launch = true

[window]
# Create/Show window at launch
create = false

# Create a transparent fullscreen (borderless) overlay window
fullscreen = true

# Enable continuous redraw
# May be used when steam oerlay detection fails or other issues with the steam overlay occur
# may increase CPU/GPU usage
continous_draw = false

[log]
# Logging level
# Allowed: "error", "warn", "info", "debug", "trace"
level = "info"

# write logs to a file
# Default (Linux): ~/.local/share/sisr/SISR.log  (or $XDG_DATA_HOME/sisr/SISR.log)
path = "C:/Users/<UserName>/AppData/Roaming/SISR/data/SISR.log"

# logging level for the file only
# If omitted, SISR uses `log.level`
file_level = "info"

[steam]
# Disable Steam CEF remote debugging
cef_debug_disable = false

# Time to wait for Steam to launch (seconds)
steam_launch_timeout_secs = 1

# explicit Steam path, normally auto inferred to something like:
steam_path = "C:/Program Files (x86)/Steam"
```
