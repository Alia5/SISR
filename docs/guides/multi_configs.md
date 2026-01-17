# 🎮 Multiple Configurations / GlosSI like usage

??? tip "For users familiar with GloSC/GlosSI "
    If you are used to GloSC/GlosSI, you may know that GlosSI "required"
    you to create a Steam shortcut to it for each game/application you wanted to use it with.

    SISR is not exactly designed like this, but it can be used in a similar manner.

    !!! warning "No Game Launching"
        Unlike GloSC/GlosSI, SISR does **not** launch games for you!  

Aside from launching SISR as a tray application you can also launch SISR from Steam directly.  
This enables you to have per Game/Application Steam Input configurations
as well as access to Touch/Radial menus and other Steam overlay features.  

!!! tip "Read general usage as well"
    I would like to encourage you to read the [General usage](./usage.md) guide as well,
    even if you don't plan to use SISR as a tray application, replacing Steams Desktop configuration.  

    The general usage guide also contains useful information and tips about SISR in general.

---

<div class="inline end">
<div class="admonition warning inline end">
<p class="admonition-title">Marker shortcut</p>
<p>You should <strong>not</strong> re-use the <em>"SISR Marker"</em> shortcut or add the <code>--marker</code> launch option here!</p>
</div>

<div class="admonition tip inline end">
<p class="admonition-title">Multiple Configurations</p>
<p>You can add SISR multiple times as non-Steam game to have multiple different Steam Input configurations available!</p>
</div>
</div>

- <h3>**1.** Add SISR as a non-Steam game in your Steam library</h3>
      Provide the following flags as launch options  
      - `-w -f`  
      <sup>show-window, fullscreen</sup>  
      If you want to use touch- and radial-menus, also add: `--wcd true`

      You can rename the shortcut to something meaningful, like _"SISR - Game XYZ"_
      and set any custom icon and images as you see fit

<br />

- <h3> **2.** Connect **all** controllers you want to use</h3>

- <h3> **3.** Launch the newly created shortcut from Steam</h3>

    This will start SISR and create a combined SISR/Steam overlay and create emulated controllers

<br />

- <h3> **4.** Launch any game or application</h3>
   Games/application **should be** launched **outside of Steam** in **most cases**  
   There are two main exceptions to this:  
     1. If the game/application does **NOT** accept the Steam Overlay / Steam Input **at all**
        Like but not limited to Windows Store games/apps
     2. Games with **native** Playstation controller support **_if_**  
        - You've set SISR to emulate Playstation controllers instead of Xbox360 controllers
        - **and** You set `Playstation Controller support` in Steam settings to `Enabled in Games w/o Support` or `Disabled`

!!! tip "SISR overlay"
    If you want to exit SISR or change settings while in-game,
    you can toggle the SISR overlay by using the keyboard-shortcut or controller-chord  
    (**`CTRL+SHIFT+ALT+S`**, **`LB+RB+BACK+A`** _"A" button needs to be pressed last_)

!!! info "Touch/Radial menus"
    By default, SISR will **not** draw continuously, which prevents touch/radial menus from showing up  
    To make touch/radial menus show up, you can add the `--wcd true` launch option to enable continuous drawing  

    This can also circumvent issues with the Steam overlay not showing up correctly.

    **Do note that this may increase CPU/GPU usage** and
    can potentially negatively affect gaming performance on lower end systems

!!! tip "Playstation Controller emulation"
    If you want SISR to emulate a Playstation controller instead of the default Xbox360 controller,
    to circumvent compatibility issues with for example Sony games, you can add the launch option:
    `--default-controller-type dualshock4`  

    You can also set this permanently in a config file or via environment variable  
    See [Configuration](../config/config.md)
