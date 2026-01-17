# 🛠️ Installation

!!! note "Steam must be running"
    Make sure Steam is running before installing/running SISR.

## 🤖 Automatic Installation Scripts

!!! warning
    The install scripts are not tested extensivly on all platforms/configurations

    If you experience problems with those scripts, open an [issue on GitHub](https://github.com/Alia5/SISR/issues)
    and follow the [Manual Installation](#-manual-installation) instructions below

SISR provides automatic installation scripts for both 🪟 Windows and 🐧 Linux
that should setup everything you need to get started with SISR quickly

=== "🪟 Windows"

    In a PowerShell run

    ```powershell
    irm https://alia5.github.io/SISR/stable/install.ps1 | iex
    ```

    The script will:

    - Download and install (or update) SISR to `%LOCALAPPDATA%\SISR\SISR.exe`
    - Download and install VIIPER
      Using the [VIIPER install script](https://alia5.github.io/VIIPER/stable/getting-started/installation/#automated-install-script) to
      `%LOCALAPPDATA%\VIIPER\viiper.exe`
        - Setup the USBIP-Win2 driver  
          <sup>Driver only, not the full USBIP-Win2 package</sup>
    - Enable Steam CEF remote debugging
    - Create Desktop and Start Menu shortcuts

    !!! tip "Version-Specific Installation"
        The install scripts are version-aware based on where you download them from:

        - **Latest stable release:**  
        `irm https://alia5.github.io/SISR/stable/install.ps1 | iex`

        - **Specific version (e.g., v0.2.2):**  
        `irm https://alia5.github.io/SISR/0.2.2/install.ps1 | iex`

        - **Latest _pre_-release (development snapshot):**  
        `irm https://alia5.github.io/SISR/main/install.ps1 | iex`

    ➡️ Continue with [Post-Installation](#i-post-installation)

=== "🐧 Linux"

    In a terminal run

    ```bash
    curl -fsSL https://alia5.github.io/SISR/stable/install.sh | sh
    ```

    The script will:

    - Download and install SISR to `~/.local/share/SISR/SISR.AppImage`
    - Download and install VIIPER as a systemd service  
      Using the [VIIPER install script](https://alia5.github.io/VIIPER/stable/getting-started/installation/#automated-install-script)
        - Attempt to setup USBIP (installation of required packages)
        - Load the required `vhci-hcd` kernel module  
          And setup automatic loading on boot
    - Enable Steam CEF remote debugging
    - Create a Launcher entry for SISR  

    !!! tip "Version-Specific Installation"
        The install scripts are version-aware based on where you download them from:

        - **Latest stable release:**  
        `curl -fsSL https://alia5.github.io/SISR/stable/install.sh | sh`

        - **Specific version (e.g., v0.2.2):**  
        `curl -fsSL https://alia5.github.io/SISR/0.2.2/install.sh | sh`

        - **Latest _pre_-release (development snapshot):**  
        `curl -fsSL https://alia5.github.io/SISR/main/install.sh | sh`

    ➡️ Continue with [Post-Installation](#i-post-installation)

## 📖 Manual Installation

### 📝 Prerequisites

- Steam installed and running (obviously...)
- A working **USBIP client** on your machine  

### 🔌 USBIP

  **READ AND FOLLOW**: [USBIP Installation](usbip.md)

### ⚙️ VIIPER

=== "Windows"

    VIIPER is bundled with SISR.No separate installation required  

    That said, for networked scenarios it's easier to install VIIPER as a standalone service using the [VIIPER install script](https://alia5.github.io/VIIPER/stable/getting-started/installation/#automated-install-script)

    In a powerShell run

    ```powershell
    irm https://alia5.github.io/VIIPER/stable/install.ps1 | iex
    ```

=== "Linux"

    Linux requires VIIPER as a system service:

    ```bash
    curl -fsSL https://alia5.github.io/VIIPER/stable/install.sh | sh
    ```

    This will install VIIPER, and run it as a systemd service.

    For more details, see the [VIIPER documentation](https://alia5.github.io/VIIPER/)

    ??? tip "Steam OS users"
        If you are installing SISR on Steam OS, you have to switch to the desktop mode and enable write access to the root filesystem first:

        ```bash
        sudo steamos-readonly disable
        ```
    

## 🚀 Getting SISR running

### 📦 Download SISR

=== "Windows"

    Download the zip archive for your architecture from the [Downloads](../downloads/index.md) page and extract it to a permanent location  

    A permanent location is important as SISR creates a marker shortcut in Steam that points to it's location

=== "Linux"

    Download the AppImage for your architecture from the [Downloads](../downloads/index.md) page
    and make it executable
    
    ```bash
    chmod +x SISR-x86_64.AppImage
    ```

    Move the file to a permanent location, this is important as SISR creates a marker shortcut in Steam that points to it's location

### 🎮 First Run

Once you have the prerequisites installed, run SISR via the Desktop or Start-Menu shortcut and follow the dialogs 😉  

Continue to [Post-Installation](#i-post-installation)

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
      - Set the shortcut's launch options to `--marker`

4. Start/Restart SISR.

## ℹ️ Post-Installation

!!! info "Marker Shortcut"

    SISR may ask you to create the "SISR Marker" shortcut in Steam (if not done already)

    SISR uses this shortcut to manage the Steam Input configuration for the emulated controllers and is required for operation  

    You can have other launch options as well, just make sure that only **a single** shortcut to SISR with the `--marker` argument exists in your Steam library.  

    You can even add multiple SISR shortcuts without `--marker` if you want to have different Steam Input configurations for different games/setups, but those will only work if those shortcuts are launched from Steam directly.  
    See the [GlosSI like usage](../guides/glossi_like.md) guide for more details

Post installation, when running SISR, it should be visible in your system tray
and your controller(s) should be available on a system level.  

If you want to change the Steam Input configuration, right click the tray icon and select "Steam Controllerconfig"
**or** change the Controller config of the `SISR Marker` shortcut in your Steam library

(Note: yes, you can rename the shortcut, just make sure to keep the `--marker` argument
and that the SISR executable is located at a stable path that does not change between runs)  

!!! tip "SISR Overlay"
    If you want to stop/start forwarding or change settings while in-game,
    you can toggle the SISR overlay from the system tray **or** by using the keyboard-shortcut or controller-chord  
    (**`CTRL+SHIFT+ALT+S`**, **`LB+RB+BACK+A`** _"A" button needs to be pressed last_)

??? info "CEF Debugging Port"

    SISR uses Steam's CEF debugging functionality located at port `8080`.  
    _Steam does not provide an easy way to permanently change this port._  
    If something else is using it (or it's blocked), SISR may not work as expected.

## ➡️ Next steps

- Checkout the [Guides](../guides/overview.md) for different usage scenarios
- Check [Configuration](../config/config.md) and [CLI Reference](../config/cli.md)
- Check the [Troubleshooting](../misc/troubleshooting.md) section
- Read the [FAQ](../misc/faq.md)
