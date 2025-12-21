# 🎮 GlosSI like usage

SISR can be used in a similar manner to GloSC/GlosSI, by creating a Steam shortcut
that launches SISR with a Steam overlay window  
This allows you to have an infinite number of different Steam Input configurations  
It also provides a combined SISR/Steam overlay window for easy access to Steam features

---

!!! warning inline end "Marker shortcut"  
    You should **not** re-use the _"SISR Marker"_ shortcut or add the `--marker` launch option here!

- Add SISR as a non-Steam game in your Steam library  
      Provide the following flags as launch options  
      - `-w -f`  
      <sup>show-window, fullscreen</sup>  
<br />

!!! tip inline end "Multiple Configurations"
    You can add SISR multiple times as non-Steam game to have multiple different Steam Input configurations available!

- Launch the newly created shortcut from Steam  
    This will start SISR and create a combined SISR/Steam overlay and create the emulated controllers

<br />

- Launch any game or application **outside of Steam**  

!!! tip "SISR overlay"
    If you want to exit SISR or change settings while in-game,
    you can toggle the SISR overlay by using the keyboard-shortcut or controller-chord  
    (**`CTRL+SHIFT+ALT+S`**, **`LB+RB+BACK+A`**)

!!! info "Touch/Radial menus"
    By default, SISR will **not** draw continuously, which prevents touch/radial menus from showing up  
    To make touch/radial menus show up, you can add the `--wcd` launch option to enable continuous drawing  

    This can also circumvent issues with the Steam overlay not showing up correctly.

    **Do note that this may increase CPU/GPU usage** and
    can potentially negatively affect gaming performance on lower end systems

!!! warning "No Game Launching"
    Unlike GloSC/GlosSI, SISR does **not** launch games for you!  
    You have to launch the game/application yourself, **outside of Steam**.
