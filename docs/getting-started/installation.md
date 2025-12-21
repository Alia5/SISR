# 🛠️ Installation

!!! warning
    There currently is no one-click installer.
    You need to manually download/install the prerequisites manually and keep the SISR binary in a **stable path** (don’t move it around after running it for the first time).

## ✅ Requirements

- Steam installed and running (obviously...)
- A working **USBIP client** on your machine  

### 🔌 USBIP

  **READ AND FOLLOW**: [USBIP Installation](usbip.md)

### ⚙️ VIIPER

=== "Linux"

    Linux requires VIIPER as a system service:

    ```bash
    curl -fsSL https://alia5.github.io/VIIPER/stable/install.sh | sh
    ```

    This will install VIIPER, and run it as a systemd service.

    For more details, see the [VIIPER documentation](https://alia5.github.io/VIIPER/).

=== "Windows"

    VIIPER is bundled with SISR. No separate installation required.

## 🚀 Getting SISR running

Once you have the prerequisites installed, run SISR and follow the dialogs 😉

### 🧰 Manual setup

!!! warning Manual Setup
    The manual setup is only required if the automatic first-time setup fails.
    which is likely as it **currently** is nothing more than a series of warning/error dialogs

1. Start Steam.

2. Enable Steam CEF remote debugging

      - Create an empty file named `.cef-enable-remote-debugging` in your
        **Steam installation directory**
        - Windows example: `C:\Program Files (x86)\Steam\.cef-enable-remote-debugging`
        - Linux example: `~/.steam/steam/.cef-enable-remote-debugging`
      - Restart Steam

3. Create the Steam "marker shortcut"

      - Add SISR as a **non-Steam game**
      - Set the shortcut’s launch options to `--marker`

4. Start/Restart SISR.

!!! info "CEF Debugging Port"

    SISR uses Steam’s CEF debugging functionality located at port `8080`.  
    _Steam does not provide an easy way to permanently change this port._  
    If something else is using it (or it’s blocked), SISR may not work as expected.

!!! info "Marker Shortcut"

    You can have other launch options as well, just make sure that only **a single** shortcut to SISR with the `--marker` argument exists in your Steam library.  
    You can even add multiple SISR shortcuts without `--marker` if you want to have different Steam Input configurations for different games/setups, but those will only work if those shortcuts are launched from Steam directly.

SISR should now be running in your system tray and your controller(s) should be available on a system level.  

If you want to change the Steam Input configuration, right click the tray icon and select "Steam Controllerconfig" **or** change the Controller config of the `SISR Marker` shortcut in your Steam library.
(Note yes, you can rename the shortcut, just make sure to keep the `--marker` argument and that the SISR executable is located at a stable path that does not change between runs)  

## ➡️ Next steps

- Check [Configuration](../config/config.md) and [CLI Reference](../config/cli.md)
- Check the [Troubleshooting](../misq/troubleshooting.md) section
- Read the [FAQ](../misq/faq.md)
