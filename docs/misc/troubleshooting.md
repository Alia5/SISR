# Troubleshooting

<style>
    .md-typeset details.question {
        border-color: rgba(128, 128, 128, 0.33);
        &:focus-within {
            box-shadow: 0 0 0 .2rem #448aff1a;
        }
        & summary {
            background: transparent;
            &::before {
                color: #227399a9;
                background-color: #227399a9;
                outline: transparent;
            }
            &::before:focus,
            &::before:focus-visible {
                outline: transparent;
                box-shadow: transparent;
            }
            &::after {
                color: var(--md-default-fg-color);
            }
        }
    }
    .toc-anchor {
        position: absolute;
        opacity: 0;
        overflow: hidden;
        width: 0;
        height: 0;
        padding: 0;
        margin: 0 !important;
        pointer-events: none;
    }
 </style>

 <script>
(()=>{
    const open=(hash)=>{
        if(!hash||hash==="#")return;
        const h=document.getElementById(hash.slice(1));
        let n=h?.nextElementSibling;
        while(n){
            if(/^H[1-6]$/.test(n.tagName)) break;
            if(n.tagName==="DETAILS"){n.open=true;break;}
            n=n.nextElementSibling;
        }
    };
    let last="";
    const tick=()=>{const h=location.hash;if(h!==last){last=h;open(h);}requestAnimationFrame(tick);};
    requestAnimationFrame(tick);
})();
</script>

<div class="grid cards" markdown>

- **Controller Issues**

    ---

    Problems with controller detection, doubling, and game compatibility

    [Jump to section](#controller-issues)

- **UI / Window Issues**

    ---

    Window visibility, overlay problems, and mouse capture

    [Jump to section](#ui-window-issues)

- **VIIPER Issues**

    ---

    Connection problems, version mismatches, and USBIP setup

    [Jump to section](#viiper-issues)

- **Steam Integration**

    ---

    Marker shortcuts, CEF debugging, and port conflicts

    [Jump to section](#steam-integration)

- **Keyboard/Mouse Emulation**

    ---

    KB/M emulation configuration and troubleshooting

    [Jump to section](#keyboard-mouse-emulation)

- **Performance**

    ---

    Input lag, latency issues, and optimization tips

    [Jump to section](#performance)

</div>

---

## 🎮 Controller Issues {#controller-issues}

### Doubled controllers / One physical controller controls multiple emulated controllers {.toc-anchor}

??? question "Doubled controllers / One physical controller controls multiple emulated controllers"

    You can try one of the two following things:

    1. Ensure that in the Steam Controller configurator for SISR,
    the controller order uses your "real" controllers **before any emulated controllers**  

    2. Use the `--require-controllers-connected-before-launch` option (_default_) to make SISR ignore controllers that are created after SISR has started

    3. Depending on what controller type you emulate  
    Turn off `Enable Steam Input for Xbox controllers` or `Playstation Controller support` in Steam settings.  
    <br />
    Otherwise Steam will pass through the emulated controller to SISR, which will then create another virtual  
    controller, which will be passed to Steam, which will it pass to SISR, which will then create another virtual  
    controller, which will be passed to Steam, which will it pass to SISR, which will then create another virtual  
    controller, which will be passed to Steam, which will it pass to SISR, which will then create another virtual  
    controller, which will be passed to Steam, which will it pass to SISR, which will then create another virtual  
    controller, which will be passed to Steam, which will it pass to SISR, which will then create another virtual  
    controller, which will be passed to Steam, which will it pass to SISR, which will then create another virtual  
    controller, which will be passed to Steam, which will it pass to SISR, which will then create another virtual  
    controller, which will be passed to Steam, which will it pass to SISR, which will then create another virtual  
    controller, which will be passed to Steam, which will it pass to SISR, which will then create another virtual  
    controller, which will be passed to Steam, which will it pass to SISR....

    !!! info "Controller identification"
        Steams "Identify Controllers" feature (available when re-ordering controller **in Steam**) will
        help you differentiate physical and emulated controllers
    
    !!! note "Playstation Controllers"
        Setting the value to "Enabled in Games w/o Support" is perfectly fine too!

### Controllers not detected / need to be connected after SISR starts {.toc-anchor}

??? question "Controllers not detected / need to be connected after SISR starts"

    By default, SISR requires controllers to be connected **before** launch to prevent (**some**) controller doubling issues.
    
    If you need to connect controllers after SISR has started, you can disable this behavior:
    
    - Add `--require-controllers-connected-before-launch false` as launch argument  
    See: [Config](../config/config.md)

### My game still detects my real PS4/DualSense/Nintendo controller {.toc-anchor}

??? question "My game still detects my real PS4/DualSense/Nintendo controller"

    Install and use [HidHide](https://github.com/nefarius/HidHide) to hide your physical controllers from games  
    Keep the visible to Steam and SISR  
    _How?_ **RTFM**...

    !!! info "HidHide setup"
        Automatic HidHide integration will maybe follow  
        soon™

### Game doesn't recognize the controller {.toc-anchor}

??? question "Game doesn't recognize the controller"

    Does the game work with regular, real, Xbox 360 controllers?  

    - If yes, you are doing it wrong  
    - If no, tough luck

### Touch/Radial menus do not work {.toc-anchor}

??? question "Touch/Radial menus do not work"

    By default, SISR will **not** draw continuously, which prevents touch/radial menus from showing up  
    To make touch/radial menus show up, you can add the `--wcd true` launch option to enable continuous drawing  

    This can also circumvent issues with the Steam overlay not showing up correctly.

    **Do note that this may increase CPU/GPU usage** and
    can potentially negatively affect gaming performance on lower end systems

### My controller doesn't work properly when SISR is running and I launch a game from Steam {.toc-anchor}

??? question "My controller doesn't work properly when SISR is running and I launch a game from Steam"

    SISR is meant as supporting-tools for games/applications **outside of Steam** that do not support Steam Input properly.

    Just disable/exit SISR before running your regular (working with Steam Input) games...

    Steam game launch detection is **not yet implemented**.

## 🪟 UI / Window issues {#ui-window-issues}

### I can't see the UI / The UI doesn't show up {.toc-anchor}

??? question "I can't see the UI / The UI doesn't show up"

    It's a system tray app. Right-click the tray icon to toggle the UI (among other things)
    **Or** launch with `-w --window-fullscreen false` to show the window at startup
    **If** the window runs **as overlay** press **`Ctrl+Shift+Alt+S`**
        or **`LB+RB+BACK+A`** (_A button needs to be pressed last_) to toggle UI visibility.

### I have toggled the UI but now I can't get rid of it {.toc-anchor}

??? question "I have toggled the UI but now I can't get rid of it"

    Press **`Ctrl+Shift+Alt+S`** or **`LB+RB+BACK+A`** (_"A" button needs to be pressed last_) again to toggle UI visibility

### My mouse is captured by the overlay and I can't interact with other windows {.toc-anchor}

??? question "My mouse is captured by the overlay and I can't interact with other windows"

    Press **`Ctrl+Shift+Alt+S`** or **`LB+RB+BACK+A`** (_"A" button needs to be pressed last_) to toggle UI visibility

## 🐍 VIIPER Issues {#viiper-issues}

### SISR says VIIPER is unavailable {.toc-anchor}

??? question "SISR says VIIPER is unavailable"

    1. Is VIIPER running?  
        Start manually: `viiper server`
    2. Is `viiper` / `viiper.exe` next to SISR?
        SISR tries to auto-start it if not already running as a service and the viiper-address is set to `localhost`
    3. Firewall blocking the connection?  
        Allow VIIPER through your firewall
    4. Correct address?  
        Default is `localhost:3242`. Change with `--viiper-address`
    5. **If** using remote VIIPER: Is the remote machine reachable?  
        Try pinging it

### VIIPER version too old {.toc-anchor}

??? question "VIIPER version too old"

    SISR enforces a minimum VIIPER version  
    VIIPER should come bundled with SISR, so this should not happen

    If you see this error, you likely use VIIPER on another machine or have VIIPER running as a service
    In any case check the [VIIPER Documentation](https://alia5.github.io/VIIPER/) for update instructions

### USBIP attach fails {.toc-anchor}

??? question "USBIP attach fails"

    Ensure you have USBIP set up correctly  
    See [USBIP setup](../getting-started/usbip.md)

## 🚂 Steam Integration {#steam-integration}

### SISR marker not found {.toc-anchor}

??? question "SISR marker not found"

    SISR reports the marker shortcut is missing.

    Create it manually:

    1. Add SISR as a **non-Steam Game** in Steam
    2. Set launch options to `--marker`
    3. Restart Steam and SISR

    See [Installation](../getting-started/installation.md)

### Port 8080 conflicts / CEF debugging is enabled, but SISR could not reach it {.toc-anchor}

??? question "Port 8080 conflicts / CEF debugging is enabled, but SISR could not reach it"

    As do other popular tools, SISR uses the CEF-Debugging option provided by Steam  
    and Valve decided to default that to port 8080 (_without an easy way to change this permanently_)

    Stop the conflicting service/program ¯\\\_(ツ)\_/¯  

### Steam installation could not be found {.toc-anchor}

??? question "Steam installation could not be found"

    Ensure Steam is installed and the installation directory exists  
    On Windows, check the registry entry for Steam  

    You can also specify the path explicitly with `--steam-path`

### Failed to create CEF debug enable file in Steam directory {.toc-anchor}

??? question "Failed to create CEF debug enable file in Steam directory"

    SISR couldn't write to the Steam directory (permissions issue, antivirus, etc.)

    Manually create the file `.cef-enable-remote-debugging` in your Steam installation directory  
    See [Installation](../getting-started/installation.md)

### Failed to restart Steam {.toc-anchor}

??? question "Failed to restart Steam"

    SISR couldn't restart Steam automatically via `steam://` URL scheme
    Restart Steam manually, then restart SISR

### SISR says the overlay notfier could not be initialized {.toc-anchor}

??? question "SISR says the overlay notfier could not be initialized"

    Just tap "_Yes_" or pass `--wcd true` as launch options.

### This doesn't work with "Steam Link" / "Remote Play" {.toc-anchor}

??? question "This doesn't work with "Steam Link" / "Remote Play""

    **The short answer:** Don't use SISR with Steam Link / Remote Play.

    The long answer: Don't use SISR with Steam Link / Remote Play.  
    Look into setting up Sunshine/Apollo and Moonlight instead.  

    Note that Sunshine/Apollo and Moonlight come with their own remote-input solution, that possibly interferes with SISR.  
    I have not yet had the time to write documentation for this  
    <sup>If you have used SISR with Sunshine/Apollo and Moonlight successfully, consider contributing to the documentation</sup>

## ⌨️🖱️ Keyboard/Mouse Emulation {#keyboard-mouse-emulation}

### KB/M emulation is disabled {.toc-anchor}

??? question "KB/M emulation is disabled"

    SISR disables KB/M emulation on **localhost/loopback** as it makes no sense there  

    To enable: Run VIIPER on a different machine and run SISR with `--viiper-address=<remote-ip>:3242 --keyboard-mouse-emulation=true`

## 🏎️ Performance {#performance}

### Input lag {.toc-anchor}

??? question "Input lag"

    Check:

    - Network latency (if using remote VIIPER): ping the host
    - System performance: CPU/GPU usage, background processes
    - Game settings: V-sync, frame rate limits

    !!! info
        USBIP/VIIPER do **not** introduce significant latency  
        See [VIIPER benchmarks](https://alia5.github.io/VIIPER/main/testing/e2e_latency/)

## Still stuck? 🙄

Open an issue on [GitHub](https://github.com/Alia5/SISR/issues) with:

- SISR version
- OS and version
- VIIPER version
- Relevant log output (`--log-level=debug`)
- Steps to reproduce

No guarantees of support, though.  
