# General usage of SISR

SISR is designed to be used a tray application that runs in the background,  
**effectivly replacing Steam's Desktop configuration with added gamepad support**.  

This allows you to use SISR with **any** game or application,
**regardless** of whether it is launched from Steam or not.

!!! danger "Under construction"
    SISR does not detect normal Steam game launches **yet!**  
    The application thus needs to be stopped manually **before** using Games/Application
    that normally work fine with Steam Input  

    **Soon™**

!!! tip "Non-Tray usage"
    If you do not want tray behaviour and instead prefer to run SISR via Steam
    with an individial Steam Input configuration per game/application,
    refer to the [Multiple Configurations / GlosSI like usage](./multi_configs.md) guide instead.

---

## Usage

!!! warning inline end "Controller connections"
    By default SISR requires all controllers to be connected before launching SISR itself.  
    This generally helps to prevent **some** potential controller duplication issue

    If you do not want this behaviour and require to have controllers connected/disconnected dynamically,
    you can disable this behavour  
    See: [Config](../../config/config)

After [installation](../getting-started/installation.md), simply launch SISR **outside of Steam**
via the Desktop or Start Menu shortcut.  

Your steam Input enabled controllers should now be redirected to emulated Xbox360 controllers
that are indistinguishable from real hardware and work with any game/application.

### Tray menu

You can access some SISR options by right-clicking the system tray icon  
![SISR System Tray Icon](../../assets/SISR-tray.png)

| Option                 | Description                                                            |
| ---------------------- | ---------------------------------------------------------------------- |
| Show UI                | Shows the SISR UI providing general information and a few more options |
| Steam Controllerconfig | Opens the Steam Controller configuration for the SISR Marker shortcut  |
| Force Controllerconfig | Forces Steam to use the Steam Input configuration of the SISR marker shortcut instead ofthe Desktop configuration (or any configuration from launched Steam games)                                    |
| Quit                   | Exits the application                                                  |

!!! tip "Tray menu"
    The tray menu is also available if you have launched SISR via Steam as described in the [Multiple Configurations / GlosSI like usage](./multi_configs.md) guide.

### SISR UI / Overlay

SISR provides a basic information and debug UI that can be accessed
by right-clicking the tray icon and selecting "Show UI"  

It shows detected controllers and their status as well as some basic information  
as well as allowing you to change a few settings on the fly  
or even allowing you to change the emulated controller type without restarting SISR

![SISR UI](../../assets/SISR-overlay.png)

!!! tip "Gamepad navigation"

    The UI is currently **only** mouse navigable, but don't worry

    You can use Steams Chord configuration to navigate the SISR UI with a gamepad as well!
    Simply **hold down** the "Steam"/"Guide"/"Playstation" button and your right-stick/trackpad will move the cursor.  
    The Right and Left Triggers act as left and right mouse buttons respectively.

### Configuration

If you want to change the default configuration of SISR, for example to demulate Playstation controller by default  
you can do by creating a config file in:  

- 🪟 Windows: `C:\Users\<UserName>\AppData\Roaming\SISR\config\SISR.toml`  
- 🐧 Linux: `$XDG_CONFIG_HOME/sisr/SISR.toml`

### Example: Emulate DualShock4 controllers by default

```toml
[controller_emulation]
default_controller_type = "dualshock4"

[steam]
# allow longer timeout for steam to launch if your computer is a potato ;P
steam_launch_timeout_secs = 5
# explicit Steam path, normally not needed and auto-detected
steam_path = "C:/Program Files (x86)/Steam"
```

For more information and full list of all options see: [Configuration](../../config/config.md)
