# 🌐 Networked usage

SISR can be used over the network to forward controllers, Keyboard/Mouse for game streaming scenarios  

This also allows you to use a Steam Deck (or similar devices)
as dedicated controllers without the need to stream the entire game

An explanation on why this can be preferable over other solutions can be found in the [FAQ](../misq/faq.md#why-would-i-want-to-use-this-instead-of-directly-using-usbipvirtualhere-to-forward-controllers)

---

!!! info "Installation"
    Installation for networked scenarios differs slightly, but is explained in this guide

## Receiving Machine (e.g. Gaming PC)

On the machine that will receive the forwarded controllers/Keyboard/Mouse (e.g. your Gaming PC)
you will not need to install SISR itself, only it's dependencies

### USBIP

Refer to the [USBIP setup guide](../getting-started/usbip.md)

### VIIPER

VIIPER comes bundled with SISR on Windows, but on Linux it must always be installed as a system service.

For networked scenarios you will have to run VIIPER on the receiving machine yourself.  

Instead of always interacting with the receiving machine yourself, VIIPER can be run as a background service
that automatically starts with the system.  

VIIPER provides [easy to use installation scripts](https://alia5.github.io/VIIPER/main/getting-started/installation/#automated-install-script)
for both Windows and Linux.  

=== "Windows"

    In a PowerShell run

    ```powershell
    irm https://alia5.github.io/VIIPER/stable/install.ps1 | iex
    ```

    This will
    
    - Install VIIPER to `%LOCALAPPDATA%\VIIPER\viiper.exe`
    - Set the VIIPER server to start automatically with Windows
    - Start the VIIPER server immediately
  
    !!! warning "USBIP not included"
        The install scripts do not install/setup USBIP.
        Make sure a USBIP-client is installed and configured before installing VIIPER.

=== "Linux"

    In a terminal run

    ```bash
    curl -fsSL https://alia5.github.io/VIIPER/stable/install.sh | sh
    ```

    This will

    - install VIIPER to `/usr/local/bin/viiper`
    - Creates systemd service: `/etc/systemd/system/viiper.service`
    - Enables and starts the VIIPER service immediately

    !!! warning "USBIP not included"
        The install scripts do not install/setup USBIP.
        Make sure a USBIP-client is installed and configured before installing VIIPER.

Refer to the [VIIPER documentation](https://alia5.github.io/VIIPER/) for more details and information

## Controller hosting machine (e.g. Steam Deck)

On the machine that will host the physical controllers/Keyboard/Mouse (e.g. your Steam Deck)
USBIP and VIIPER are not required, only SISR itself

!!! warning inline end "Marker shortcut"  
    You should **not** re-use the _"SISR Marker"_ shortcut  or add the `--marker` launch option here!

- Add SISR as a non-Steam game in your Steam library  
      Provide the following flags as launch options  
      - `-w -f --wcd --kbm --viiper-address <RECEIVING_MACHINE_IP>:3242`  
      <sup>show-window, fullscreen, continuous drawing, keyboard/mouse emulation, other machine address</sup>  
<br />

!!! tip inline end "Multiple Configurations"
    You can add SISR multiple times as non-Steam game to have multiple different Steam Input configurations available!

- Launch the newly created shortcut from Steam  
    This will start SISR and create virtual controllers, as well as forward Keyboard/Mouse inputs to the receiving machine

<br />
<br />

!!! tip "SISR overlay"
    If you want to stop/start forwarding or change settings while in-game,
    you can toggle the SISR overlay by using the keyboard-shortcut or controller-chord  
    (**`CTRL+SHIFT+ALT+S`**, **`LB+RB+BACK+A`**)

!!! info "Flatpak"
    There will (_hopefully_) be a Flatpak version of SISR available soon™  

    I personally do not run SteamOS myself and cannot provide more detailed instructions at this time.  

    That said, my source-built version of SISR works perfectly fine on my Arch (btw) install on my Deck, even from Valves Gamescope (Gaming mode)
