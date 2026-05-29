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
        <div><code>--config &lt;FILE&gt;</code></div>

      </td>
      <td>Path to an explicit configuration file (TOML/YAML/JSON)</td>
      <td></td>
    </tr>

    <tr>
      <td><code>--console</code> (Windows only)</td>
      <td>Show console window</td>
      <td><code>false</code></td>
    </tr>

    <tr>
      <td><code>--api.listen-address &lt;host:port&gt;</code></td>
      <td>API listen address; by default will pick a random free port and listen on localhost only</td>
      <td><code>localhost:0</code></td>
    </tr>

    <tr>
      <td><code>--api.cors-origins &lt;origins&gt;</code></td>
      <td>CORS allowed origins</td>
      <td><code>https://steamloopback.host,http://steamloopback.host</code></td>
    </tr>

    <tr>
      <td><code>--viiper.address &lt;host:port&gt;</code></td>
      <td>VIIPER server address; specify if VIIPER is run manually or on another machine</td>
      <td><code>localhost:3242</code></td>
    </tr>

    <tr>
      <td><code>--viiper.password &lt;password&gt;</code></td>
      <td>VIIPER server password. Required for non-localhost connections</td>
      <td><i>none</i></td>
    </tr>

    <tr>
      <td><code>--keyboard-mouse-emulation</code>, <code>--kbm</code></td>
      <td>Forward keyboard and mouse when running over a network</td>
      <td><code>false</code></td>
    </tr>

    <tr>
      <td><code>--update-notify &lt;CHANNEL&gt;</code></td>
      <td>Update notification level (<code>none</code>, <code>stable</code>, <code>prerelease</code>, <code>always</code>)</td>
      <td><code>stable</code></td>
    </tr>

    <tr>
      <td><code>--default-controller-type &lt;type&gt;</code></td>
      <td>Default controller type for emulation (<code>xbox360</code>, <code>dualshock4</code>)</td>
      <td><code>xbox360</code></td>
    </tr>

    <tr>
      <td><code>--gyro-passthrough</code></td>
      <td>Enable gyro passthrough for supported controllers</td>
      <td><code>true</code></td>
    </tr>

    <tr>
      <td><code>--allow-steam-desktop-layout</code></td>
      <td>Allow/use Steam's desktop configuration for emulated controllers</td>
      <td><code>false</code></td>
    </tr>

    <tr>
      <td><code>--immediate-sensor-updates</code></td>
      <td>Immediately send sensor updates to VIIPER instead of waiting for the next input report</td>
      <td><code>true</code></td>
    </tr>

    <tr>
      <td>
        <div><code>--w</code></div>
        <div><code>--window.show [true|false]</code></div>
      </td>
      <td>Shows window on startup; when used with fullscreen enabled, this will enable Steam Overlay</td>
      <td><code>false</code></td>
    </tr>

    <tr>
      <td>
        <div><code>--f</code></div>
        <div><code>--window.fullscreen [true|false]</code></div>
      </td>
      <td>Create a transparent, borderless, always-on-top overlay window</td>
      <td><code>true</code></td>
    </tr>

    <tr>
      <td><code>--window.max-fps &lt;FPS&gt;</code></td>
      <td>Maximim FPS for SteamOverlay/UI (Does not affect inputs)</td>
      <td><code>60</code></td>
    </tr>

    <tr>
      <td><code>--log.level &lt;LEVEL&gt;</code></td>
      <td>Log level (<code>trace</code>, <code>debug</code>, <code>info</code>, <code>warn</code>, <code>error</code>)</td>
      <td><code>info</code></td>
    </tr>

    <tr>
      <td><code>--log.file &lt;FILE&gt;</code></td>
      <td>Write logs to a file</td>
      <td><i>none</i></td>
    </tr>

    <tr>
      <td><code>--no-steam</code></td>
      <td>Run in no-Steam mode</td>
      <td><code>false</code></td>
    </tr>

    <tr>
      <td><code>--steam.install-dir &lt;PATH&gt;</code></td>
      <td>Steam installation directory (optional, will attempt to auto-detect if not set)</td>
      <td><i>auto-detect</i></td>
    </tr>

    <tr>
      <td><code>--steam.user-id &lt;ID&gt;</code></td>
      <td>Active Steam user ID (optional, will attempt to auto-detect if not set)</td>
      <td><i>auto-detect</i></td>
    </tr>

    <tr>
      <td><code>--steam.cef-remote-debug-port &lt;PORT&gt;</code></td>
      <td>CEF remote debugging port (optional, will attempt to auto-detect if not set)</td>
      <td><i>auto-detect</i></td>
    </tr>
  </tbody>
</table>

## Config file discovery

SISR looks for these names and extensions in the current working directory:

- Names: `SISR`, `config`
- Extensions: `.toml`, `.yaml`, `.yml`, `.json`

Search locations:

- The current working directory

You can also explicitly provide a config file path via `--config <FILE>` or the `SISR_CONFIG` environment variable.

### Discovery order and precedence

When multiple sources provide the same configuration option, the latest one in the following is picked:

1. Defaults
2. Discovered config files (in this exact order):  
     1. `github.com/Alia5/SISR.{json,yaml,yml,toml}`
     2. `SISR.{json,yaml,yml,toml}`
     3. `config.{json,yaml,yml,toml}`
3. Your explicit `--config <FILE>` (if provided)
4. CLI flags and environment variables

## Full example (default) configuration

All values are optional.

🪟 `%APPDATA%\SISR\config\SISR.toml`  
🐧 `$XDG_CONFIG_HOME/sisr/SISR.toml`

```toml
# Windows only: show console window
# On other platforms this key is ignored
console = false

# API server address; 0 picks a random free port
[api]
listen-address = "localhost:0"

# Optional CORS origins
cors-origins = "https://steamloopback.host,http://steamloopback.host"

# VIIPER API server address
[viiper]
address = "localhost:3242"

# VIIPER API server password
password = ""

# Enable keyboard/mouse emulation.
# Will only work if the specified VIIPER server does not run on localhost
keyboard-mouse-emulation = false

# Update notification channel
# Allowed: "none", "stable", "prerelease", "always"
update-notify = "stable"

# Default controller type for emulation
default-controller-type = "xbox360"

# Allow/re-use Steam's desktop configuration for emulated controllers
allow-steam-desktop-layout = false

# Enable gyro passthrough for supported controllers
gyro-passthrough = true

# Send sensor updates immediately instead of waiting for the next input report
immediate-sensor-updates = true

# No-Steam mode
no-steam = false

# Window behavior
[window]
# Create/Show window at launch
show = false

# Create a transparent fullscreen (borderless) overlay window
fullscreen = true

# Maximum FPS for the Steam overlay/UI
max-fps = 60

# Logging
[log]
# Logging level
# Allowed: "trace", "debug", "info", "warn", "error"
level = "info"

# Write logs to a file.
# Default path:
#   Windows: %APPDATA%/SISR/data/SISR.log
#   Linux: ~/.config/SISR/data/SISR.log
file = "C:/Users/<UserName>/AppData/Roaming/SISR/data/SISR.log"

# Steam integration
[steam]
# Steam installation directory, normally auto-detected
install-dir = "C:/Program Files (x86)/Steam"

# Active Steam user ID (optional)
user-id = 0

# CEF remote debugging port (optional)
cef-remote-debug-port = 8080
```
