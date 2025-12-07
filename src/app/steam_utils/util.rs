use std::{path::PathBuf, process::Command, sync::OnceLock};
use tracing::debug;
use tracing::trace;
use tracing::warn;

static STEAM_PATH: OnceLock<Option<PathBuf>> = OnceLock::new();
static LAUNCHED_VIA_STEAM: OnceLock<bool> = OnceLock::new();

pub fn init() {
    let launched_via_steam = std::env::var("SteamGameId").is_ok();
    LAUNCHED_VIA_STEAM.set(launched_via_steam).ok();
    debug!("Launched via Steam: {}", launched_via_steam);
}

pub fn launched_via_steam() -> bool {
    *LAUNCHED_VIA_STEAM.get().unwrap_or(&false)
}

pub fn open_steam_url(url: &str) -> Result<(), std::io::Error> {
    debug!("Opening Steam URL: {}", url);

    #[cfg(target_os = "windows")]
    {
        Command::new("cmd").args(["/c", "start", "", url]).spawn()?;
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(url).spawn()?;
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open").arg(url).spawn()?;
    }

    Ok(())
}

pub fn steam_path() -> Option<PathBuf> {
    if let Some(cfg_path) = crate::config::CONFIG
        .get()
        .and_then(|cfg| cfg.steam.steam_path.clone())
    {
        trace!("Using configured Steam path: {}", cfg_path.display());
        return Some(cfg_path);
    }

    // Let's just assume steam path install doesn't change during runtime...
    if let Some(cached_path) = STEAM_PATH.get() {
        return cached_path.clone();
    }

    #[cfg(target_os = "windows")]
    {
        use winreg::RegKey;
        use winreg::enums::HKEY_CURRENT_USER;

        let hklm = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok(steam_key) = hklm.open_subkey("Software\\Valve\\Steam") {
            let Ok(install_path) = steam_key.get_value("SteamPath") as Result<String, _> else {
                return None;
            };
            let path = Some(PathBuf::from(install_path));
            trace!(
                "Found Steam install path {}",
                path.as_ref().unwrap().display()
            );
            STEAM_PATH.set(path.clone()).ok();
            return path;
        }
        None
    }
    #[cfg(target_os = "linux")]
    {
        if let Some(home_dir) = directories::BaseDirs::new().map(|bd| bd.home_dir().to_path_buf()) {
            let steam_path = home_dir.join(".steam/steam");
            if steam_path.exists() {
                let path = Some(steam_path);
                trace!(
                    "Found Steam install path {}",
                    path.as_ref().unwrap().display()
                );
                STEAM_PATH.set(path.clone()).ok();
                return path;
            }
        }
        None
    }
    #[cfg(target_os = "macos")]
    {
        if let Some(home_dir) = directories::BaseDirs::new().map(|bd| bd.home_dir().to_path_buf()) {
            let steam_path = home_dir.join("Library/Application Support/Steam");
            if steam_path.exists() {
                let path = Some(steam_path);
                trace!(
                    "Found Steam install path {}",
                    path.as_ref().unwrap().display()
                );
                STEAM_PATH.set(path.clone()).ok();
                return path;
            }
        }
        None
    }
}

pub fn steam_running() -> bool {
    use sysinfo::System;

    let mut system = System::new_all();
    system.refresh_all();
    if system.processes().is_empty() {
        warn!("Failed to get process list to check for Steam process");
        return false;
    }

    for process in system.processes().values() {
        #[cfg(target_os = "windows")]
        {
            if process.name().to_str().unwrap_or_default() == "steam.exe" {
                return true;
            }
        }
        #[cfg(target_os = "linux")]
        {
            if process.name().to_str().unwrap_or_default() == "steam" {
                return true;
            }
        }
        #[cfg(target_os = "macos")]
        {
            if process.name().to_str().unwrap_or_default() == "steam_osx" {
                return true;
            }
        }
    }
    false
}
