use std::{path::PathBuf, process::Command};
use tracing::debug;

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
    if let Some(home_dir) = directories::BaseDirs::new().map(|bd| bd.home_dir().to_path_buf()) {
        tracing::debug!("Home directory found: {}", home_dir.display());
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
            return Some(PathBuf::from(install_path));
        }
        None
    }
    #[cfg(target_os = "linux")]
    {
        if let Some(home_dir) = directories::BaseDirs::new().map(|bd| bd.home_dir().to_path_buf()) {
            let steam_path = home_dir.join(".steam/steam");
            if steam_path.exists() {
                return Some(steam_path);
            }
        }
        None
    }
    #[cfg(target_os = "macos")]
    {
        if let Some(home_dir) = directories::BaseDirs::new().map(|bd| bd.home_dir().to_path_buf()) {
            let steam_path = home_dir.join("Library/Application Support/Steam");
            if steam_path.exists() {
                return Some(steam_path);
            }
        }
        None
    }
}

pub fn steam_running() -> bool {
    #[cfg(target_os = "windows")]
    {
        use sysinfo::System;

        let mut system = System::new_all();
        system.refresh_all();

        for (_pid, process) in system.processes() {
            if process.name().to_str().unwrap_or_default() == "steam.exe" {
                return true;
            }
        }
        false
    }
    #[cfg(target_os = "linux")]
    {
        use sysinfo::System;

        let mut system = System::new_all();
        system.refresh_all();

        for (_pid, process) in system.processes() {
            if process.name().to_str().unwrap_or_default() == "steam" {
                return true;
            }
        }
        false
    }
    #[cfg(target_os = "macos")]
    {
        use sysinfo::System;

        let mut system = System::new_all();
        system.refresh_all();

        for (_pid, process) in system.processes() {
            if process.name().to_str().unwrap_or_default() == "steam_osc" {
                return true;
            }
        }
        false
    }
}
