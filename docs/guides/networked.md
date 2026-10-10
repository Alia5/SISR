# 🌐 Networked usage

SISR can be used over the network to forward controllers, Keyboard/Mouse for game streaming scenarios  

This also allows you to use a Steam Deck (or similar devices)
as dedicated controllers without the need to stream the entire game

SISR also forwards any keyboard/mouse inputs _(if `--kbm=true` is passed as launch argument)_
This includes inputs from physical keyboards/mice connected to the hosting machine,
as well as Steam Input keyboard/mouse bindings and the Steam on-screen keyboard

!!! note "No direct Deck passthrough"
    SISR does **not** directly passthrough the Decks built-in controller hardware  
    (or any direct controller passthrough, for that matter)  
    Instead, SISR forwards Steams Virtual Controllers created from the Decks inputs
    as Xbox360 controllers and a dedicated Keyboard/Mouse device.  

    While still not perfect, this approach circumvents many issues present in direct USBIP/VirtualHere passthrough solutions.

An explanation on why this can be preferable over other solutions can be found in the [FAQ](../misc/faq.md#why-would-i-want-to-use-this-instead-of-directly-using-usbipvirtualhere-to-forward-controllers)

---

!!! info "Installation"
    Installation for networked scenarios differs slightly, but is explained in this guide

## Receiving Machine (e.g. Gaming PC)

On the machine that will receive the forwarded controllers/Keyboard/Mouse (e.g. your Gaming PC)
you will not need to install SISR itself, only it's dependencies, USBIP and VIIPER

### USBIP

Refer to the [USBIP setup guide](../getting-started/usbip.md)

!!! tip "SISR previously setup"
    If you have previously installed SISR on the receiving machine,
    USBIP should already be setup correctly and you can skip this step

### VIIPER

!!! tip "SISR previously setup"
    If you have previously installed SISR on the receiving machine,
    VIIPER should already be setup correctly and you can skip this step

For networked scenarios you will have to install [VIIPER](https://alia5.github.io/VIIPER/) as a background "service".  
_(VIIPER is used by SISR to emulate controllers!)_

The VIIPER executable is its own
[installer](https://alia5.github.io/VIIPER/stable/getting-started/installation/#installer).  

=== "Windows"

    In a PowerShell run

    ```powershell
    winget install Alia5.VIIPER
    ```

    This will
    
    - Install VIIPER to `%LOCALAPPDATA%\VIIPER\viiper.exe`
    - Set the VIIPER server to start automatically with Windows
    - Install USBIP-Win2
    - Start the VIIPER server immediately

=== "Linux"

    Download `viiper-linux-amd64` from the [VIIPER releases](https://github.com/Alia5/VIIPER/releases) and in a terminal run

    ```bash
    chmod +x viiper-linux-amd64
    sudo ./viiper-linux-amd64 install
    ```

    This will

    - install VIIPER to `/usr/local/bin/viiper`
    - Creates systemd service: `/etc/systemd/system/viiper.service`
    - Installs USBIP and loads the `vhci_hcd` kernel module
    - Enables and starts the VIIPER service immediately

Refer to the [VIIPER documentation](https://alia5.github.io/VIIPER/) for more details and information

### Authentication/Password

VIIPER **requires** a password to connect (from non-localhost) for security reasons.  

You can find or change the password (on the machine running VIIPER) in:

- **Windows**:               `%APPDATA%\\VIIPER\\viiper.key.txt`
- **Linux (user)**:    `~/.config/github.com/Alia5/viiper/viiper.key.txt`
- **Linux (root/systemd)**:  `/etc/viiper/viiper.key.txt`

You can also set a custom password by creating/modifying the file above **and restarting the VIIPER service**

## Controller hosting machine (e.g. Steam Deck)

On the machine that will host the physical controllers/Keyboard/Mouse (e.g. your Steam Deck)
USBIP and VIIPER are not required, only SISR itself

!!! warning inline end "Marker shortcut"  
    You should **not** re-use the _"SISR Marker"_ shortcut  or add the `--marker` launch option here!

- Add SISR as a non-Steam game in your Steam library  
      Provide the following flags as launch options  
    - `-w --f --kbm true --viiper.address <RECEIVING_MACHINE_IP>:3242 --viiper.password <VIIPER_PASSWORD>`  
    <sup>show-window, fullscreen, keyboard/mouse emulation, other machine address</sup>  
<br />

??? info "Click to reveal screenshot"
    ![Example Deck Controller Shortcut](../assets/deck-controller-shortcut.png)  
    Example Steam shortcut for using a Steam Deck as a dedicated controller

<br />

!!! tip inline end "Multiple Configurations"
    You can add SISR multiple times as non-Steam game to have multiple different Steam Input configurations available!

- Launch the newly created shortcut from Steam  
    This will start SISR and create virtual controllers, as well as forward Keyboard/Mouse inputs to the receiving machine

<br />
<br />

!!! info "Multiple device forwarding"
    SISR will not only forward the Steam Decks built-in controller,
    but any other controller that is connected to the Deck (or other PC) as well!  
    (As long as the controller is picked up by Steam)

!!! tip "SISR overlay"
    If you want to stop/start forwarding or change settings while in-game,
    you can toggle the SISR overlay by using the keyboard-shortcut or controller-chord  
    (**`CTRL+SHIFT+ALT+S`**, **`LB+RB+BACK+A`** (_"A" button needs to be pressed last_))
