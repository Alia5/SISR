# FAQ

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

## Where's the GUI? {.toc-anchor}

??? question "Where's the GUI?"

    SISR runs as a **system tray application** by default.

    - Right-click the tray icon to show/hide the UI
    - Or launch with `-w --window-fullscreen false` to show the window at startup
    - **If** the window runs **as overlay** press **`Ctrl+Shift+Alt+S`**
    or **`LB+RB+BACK+A`** (_"A" button needs to be pressed last_) to toggle UI visibility.

    You can also run `sisr --help` to see all CLI options

## I don't like the system tray app behavior, can I run it as a normal windowed application? {.toc-anchor}

??? question "I don't like the system tray app behavior, can I run it as a normal windowed application?"

    Yes, you can!  
    Launch SISR with `-w --window-fullscreen false` to show the window at startup and disable fullscreen behavior

## I don't like the chosen overlay shortcut/controller chord, can I change it? {.toc-anchor}

??? question "I don't like the chosen overlay shortcut/controller chord, can I change it?"

    You do realize Steam Input can remap controller buttons/shortcuts, right?  
    You're smart, you can figure it out 😜

## Why would I use this over Steam Input directly? {.toc-anchor}

??? question "Why would I use this over Steam Input directly?"

    SISR can be used to circumvent issues with games and applications that
    do not support Steam Input or otherwise pose challenges, like (but not limited to):

    - Games with aggressive anti-cheat systems
    - Emulators
    - Windows Store games/apps
    - Games with broken Steam Input support

    You can also use SISR to "tunnel"/forward
    Steam Input configurations over the network to other machines, including Keyboard/Mouse.  

    This makes it possible to use devices like a Steam Deck as a dedicated controller
    without the need to stream the entire game

    !!! info
        That said, if you do not have issues with Steam Input directly, you probably should not use SISR at all 😉

## Can I use this with "Steam Link" / "Remote Play" {.toc-anchor}

??? question "Can I use this with "Steam Link" / "Remote Play"?"

    Theoretically(?)
    But don't expect it to also stream your game/display.

    Take a look at the [Networked usage guide](../guides/networked.md) for more details

    **If** you want game/display-streaming as well, you should  
    look into setting up Sunshine/Apollo and Moonlight   
    Note that Sunshine/Apollo and Moonlight come with their own remote-input solution, that possibly interferes with SISR.  
    I have not yet had the time to write documentation for this  
    <sup>If you have used SISR with Sunshine/Apollo and Moonlight successfully, consider contributing to the documentation</sup>

## How does this compare to Sunshine/Apollo's "input only mode" {.toc-anchor}

??? question "How does this compare to Sunshine/Apollo's _input only mode_"

    Sunshine/Apollo/WhateverTheFuck use the good old, but _unmaintained(!)_ ViGEm driver (on Windows)  
    SISR uses a different underlying tool [VIIPER](https://alia5.github.io/VIIPER/stable/) for both controller as well as keyboard/mouse inputs.

    At the **current** stage, differences (to an input-only mode) are _mostly_ technical to an end-user... however:

    - SISR does not use _unmaintained_ technologies, duh.
    - Uses the same input-emulation technique across platforms
    - SISRs controller devices are (not yet, lol) blocked by any anticheat (as opposed to ViGEms)
    - Emulates mouse/keyboard in a way that is indistinguishable from real hardware, too!
    - SISR supports emulating a DS4 controller (as does ViGEm), but with **full gyro-passthrough**  
    Touchpad passthrough will follow.  
    You can even decide this **per device**  
    - SISR will (_at a later stage_, probably™️) support Steam Controller 2 (_"Triton"_) re-emulation, allowing you to have full Steam-Input functionality on the controller receiving machine, not only on the controller-host (eg. Deck)
    - Maybe more to come.

    And of course, SISR is **only** concerned about controllers/input-devices - no display-streaming baggage needed.  
    SISR is primarily a tool to circumvent Steam Input issues, after all.

## Why would I want to use this instead of _directly_ using USBIP/VirtualHere to forward controllers? {.toc-anchor}

??? question "Why would I want to use this instead of _directly_ using USBIP/VirtualHere to forward controllers?"

    Diretly forwarding the Steam Decks (or similar devices) inputs via USBIP/VirtualHere comes with significant drawbacks:

    - The forwarded device is entirely "removed" from the host machine  
        - You cannot use the device on the host machine at all while it's forwarded
        - There may be no way to "exit" the forwarding on the host machine without additional input devices or remote access,
        you have to disconnect the USBIP/VirtualHere session from the client side  
        This can be especially problematic if the host machine is a Steam Deck...
    - USBIP on Windows specifically does not _currently_ work with the Steam Decks built-in controller
    - VirtualHere is closed-source and requires a paid license for more than a single device
    - I have personally experienced significant latency and input issues
    when using VirtualHere specifically on long gaming sessions
    - On the Steam Deck (or similar devices), the Touchscreens inputs cannot be forwarded via USBIP/VirtualHere at all

    SISR circumvents all of these issues by:

    - Forwarding Steams Virtual Controllers instead of the physical device
    - Forward additional Keyboard/Mouse devices alongside the controllers
    - Providing "_an out_" using it's dedicated shortcut/controller chord (**`CTRL+SHIFT+ALT+S`**, **`LB+RB+BACK+A`**)
    to toggle the UI visibility and stop/start forwarding
    - Being fully open-source and free to use  
    Even if a feature is not supported, everyone can grab the source-code
    of any part in the chain (except Steam itself) and implement it 😉

## What is USBIP? {.toc-anchor}

??? question "What is USBIP?"

    **USBIP** is a protocol for tunneling USB devices over TCP/IP  
    It allows a USB device on one machine to appear on another machine over the network (or localhost).

    SISR uses USBIP (via [VIIPER](https://alia5.github.io/VIIPER/)) to create emulated controllers
    that appear as real hardware at the system level

    See [USBIP setup](../getting-started/usbip.md) for setup instructions

## What is VIIPER? {.toc-anchor}

??? question "What is VIIPER?"

    VIIPER (**V**irtual **I**nput over **IP** **E**mulato**R**) is the USBIP server that SISR uses to emulate controllers.

    **VIIPER is bundled with SISR**  
    you don't need to download/setup it separately  

    VIIPER listens on:

    - `:3241` for USBIP connections
    - `:3242` for the control API

    See the [VIIPER documentation](https://alia5.github.io/VIIPER/) for more details

## Common Issues

For common issues (doubled controllers, Steam CEF debugging, port conflicts, etc.), see: [Troubleshooting](troubleshooting.md)

## I want feature XYZ

Check [GitHub Issues](https://github.com/Alia5/SISR/issues) to see if it's already requested  
If not, open a new issue  

No guarantees, though.  

Better yet, implement it yourself and open a pull request 😉  
Alternatively, you can hire me to implement it for you 😜  
Rates start at 100€/hour.
