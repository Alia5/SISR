use new_vdf_parser::open_shortcuts_vdf;
use std::sync::RwLock;
use std::{path::PathBuf, process::Command, sync::OnceLock};
use tracing::debug;
use tracing::error;
use tracing::info;
use tracing::trace;
use tracing::warn;

use crate::app::steam_utils::cef_debug;
use crate::config::CONFIG;

static STEAM_PATH: OnceLock<Option<PathBuf>> = OnceLock::new();
static LAUNCHED_VIA_STEAM: OnceLock<bool> = OnceLock::new();
static OVERLAY_LIB: RwLock<Option<libloading::Library>> = RwLock::new(None);

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
    if let Some(cfg_path) = CONFIG
        .read()
        .ok()
        .and_then(|c| c.as_ref().and_then(|cfg| cfg.steam.steam_path.clone()))
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

pub fn active_user_id() -> Option<u32> {
    #[cfg(target_os = "windows")]
    {
        use winreg::RegKey;
        use winreg::enums::HKEY_CURRENT_USER;

        let hklm = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok(steam_key) = hklm.open_subkey("Software\\Valve\\Steam\\ActiveProcess") {
            let Ok(user_id) = steam_key.get_value("ActiveUser") as Result<u32, _> else {
                return None;
            };
            trace!("Found active Steam user ID: {}", user_id);
            return Some(user_id);
        }
    }
    #[cfg(target_os = "linux")]
    {
        // Untested AI code, but wel'll see...
        if let Some(steam_path) = steam_path() {
            let registry_vdf = steam_path.parent().map(|p| p.join("registry.vdf"));
            if let Some(ref vdf_path) = registry_vdf
                && vdf_path.exists()
                && let Ok(content) = std::fs::read_to_string(vdf_path)
            {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("\"ActiveUser\"") {
                        let parts: Vec<&str> = trimmed.split('"').collect();
                        if parts.len() >= 4
                            && let Ok(user_id) = parts[3].parse::<u32>()
                            && user_id != 0
                        {
                            trace!("Found active Steam user ID from registry.vdf: {}", user_id);
                            return Some(user_id);
                        }
                    }
                }
            }
            let userdata_path = steam_path.join("userdata");
            if userdata_path.exists()
                && let Ok(entries) = std::fs::read_dir(&userdata_path)
            {
                for entry in entries.flatten() {
                    if entry.path().is_dir()
                        && let Some(name) = entry.file_name().to_str()
                        && let Ok(user_id) = name.parse::<u32>()
                        && user_id != 0
                    {
                        trace!(
                            "Found possibly active Steam user ID from userdata directory: {}",
                            user_id
                        );
                        return Some(user_id);
                    }
                }
            }
        }
    }

    None
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

pub fn get_shortcuts_path(steam_path: &PathBuf, steam_active_user_id: u32) -> Option<PathBuf> {
    let joined_path: PathBuf = steam_path
        .clone()
        .join("userdata")
        .join(steam_active_user_id.to_string())
        .join("config/shortcuts.vdf");

    if joined_path.exists() {
        Some(joined_path)
    } else {
        None
    }
}

fn get_case_insensitive<'a>(
    obj: &'a serde_json::Value,
    key: &str,
) -> Option<&'a serde_json::Value> {
    let obj_map = obj.as_object()?;
    let key_lower = key.to_lowercase();
    obj_map
        .iter()
        .find(|(k, _)| k.to_lowercase() == key_lower)
        .map(|(_, v)| v)
}

pub fn shortcuts_has_sisr_marker(shortcuts_path: &PathBuf) -> u32 {
    let shortcuts = open_shortcuts_vdf(shortcuts_path);
    trace!("Parsed shortcuts.vdf: {:?}", shortcuts);
    let running_executable_path = std::env::var("APPIMAGE")
        .ok()
        .and_then(|p| std::path::PathBuf::from(p).canonicalize().ok())
        .or_else(|| std::env::current_exe().ok())
        .unwrap_or_default();
    let running_path_str = running_executable_path
        .to_str()
        .unwrap_or_default()
        .to_lowercase();
    debug!("Current running executable path: {}", running_path_str);
    if let Some(shortcuts_array) = shortcuts.as_object() {
        for (_key, shortcut) in shortcuts_array {
            let Some(path) = get_case_insensitive(shortcut, "exe") else {
                continue;
            };
            let Some(args) = get_case_insensitive(shortcut, "LaunchOptions") else {
                continue;
            };
            let Some(path_str) = path.as_str() else {
                continue;
            };
            let Some(args_str) = args.as_str() else {
                continue;
            };
            trace!("Checking shortcut - Path: {}, Args: {}", path_str, args_str);
            if path_str
                .to_lowercase()
                .replace("\\", "/")
                .contains(&running_path_str.to_lowercase().replace("\\", "/"))
                && args_str.to_lowercase().contains("--marker")
            {
                let app_id = get_case_insensitive(shortcut, "appid")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32;
                return app_id;
            }
        }
    }
    0
}

pub async fn create_sisr_marker_shortcut() -> anyhow::Result<u32> {
    let exe_path = std::env::var("APPIMAGE")
        .ok()
        .or_else(|| {
            std::env::current_exe()
                .ok()
                .and_then(|p| p.to_str().map(String::from))
        })
        .unwrap_or_default();
    let payload = format!("var SISR_PATH = `{}`;\n", exe_path.replace("\\", "/"))
        + str::from_utf8(cef_debug::payloads::CREATE_MARKER_SHORTCUT)
            .expect("Failed to convert create marker shortcut payload to string");
    match cef_debug::inject::inject("SharedJSContext", &payload).await {
        Ok(result) => {
            debug!("Create SISR marker shortcut result: {}", result);
            let app_id: u32 = result.parse().unwrap_or(0);
            if app_id != 0 {
                Ok(app_id)
            } else {
                Err(anyhow::anyhow!(
                    "Failed to create SISR marker shortcut, invalid App ID returned"
                ))
            }
        }
        Err(e) => Err(anyhow::anyhow!(
            "Failed to create SISR marker shortcut: {}",
            e
        )),
    }
}

pub fn load_steam_overlay() {
    if launched_via_steam() {
        debug!("Launched via Steam; skipping Steam overlay load");
        return;
    }

    if !steam_running() {
        warn!("Steam is not running; cannot loading Steam overlay is useless!");
        return;
    }

    let steam_path = match steam_path() {
        Some(path) => path,
        None => {
            warn!("Could not determine Steam installation path; cannot load Steam overlay");
            return;
        }
    };

    let mut steam_overlay_path = steam_path;
    #[cfg(target_os = "windows")]
    {
        steam_overlay_path = steam_overlay_path.join("GameOverlayRenderer64.dll");
    }
    #[cfg(target_os = "linux")]
    {
        steam_overlay_path = steam_overlay_path
            .parent()
            .unwrap()
            .join("bin64")
            .join("gameoverlayrenderer.so");
    }

    unsafe {
        match libloading::Library::new(steam_overlay_path) {
            Ok(lib) => {
                OVERLAY_LIB
                    .write()
                    .expect("Couldn't lock gameoverlaystorage for writing")
                    .replace(lib);
                info!("Successfully loaded Steam overlay library");
            }
            Err(e) => {
                error!("Failed to load Steam overlay library: {}", e);
            }
        }
    }
}

pub fn unload_steam_overlay() {
    OVERLAY_LIB
        .write()
        .expect("Couldn't lock gameoverlaystorage for writing")
        .take();
    info!("Unloaded Steam overlay library");
}

pub fn try_set_marker_steam_env() -> anyhow::Result<()> {
    let Some(steam_path) = steam_path() else {
        warn!("Steam path could not be determined; Steam integration may not work correctly");
        return Err(anyhow::anyhow!("Steam path could not be determined"));
    };
    let Some(steam_active_user_id) = active_user_id() else {
        warn!(
            "Active Steam user ID could not be determined; Steam integration may not work correctly"
        );
        return Err(anyhow::anyhow!(
            "Active Steam user ID could not be determined"
        ));
    };
    let Some(shortcuts_path) = get_shortcuts_path(&steam_path.clone(), steam_active_user_id) else {
        warn!("Failed to determine Steam shortcuts.vdf path");
        return Err(anyhow::anyhow!(
            "Failed to determine Steam shortcuts.vdf path"
        ));
    };
    trace!("Steam shortcuts.vdf path: {:?}", shortcuts_path);
    let marker_app_id = shortcuts_has_sisr_marker(&shortcuts_path);
    if marker_app_id == 0 {
        warn!(
            "No SISR marker shortcut found in Steam shortcuts; Steam integration may not work correctly"
        );
        return Err(anyhow::anyhow!(
            "No SISR marker shortcut found in Steam shortcuts"
        ));
    }
    unsafe {
        std::env::set_var("SteamClientLaunch", "0");

        std::env::set_var("SteamAppId", "0");
        std::env::set_var("SISR_MARKER_ID", marker_app_id.to_string());
        let game_id = (marker_app_id as u64) << 32 | (2 << 24) as u64;
        std::env::set_var("SteamGameId", game_id.to_string());
        std::env::set_var("SteamOverlayGameId", game_id.to_string());
        // TODO: is this needed? decode the values
        // std::env::set_var("EnableConfiguratorSupport", "4111");
        std::env::set_var("SteamPath", steam_path.to_string_lossy().to_string());

        // TODO: is this always the same, and always existing?
        let gamepad_info_path = steam_path
            .clone()
            .join("config")
            .join("virtualgamepadinfo.txt");
        if !gamepad_info_path.exists() {
            warn!(
                "Steam virtualgamepadinfo.txt not found at expected path: {}",
                gamepad_info_path.display()
            );
            return Err(anyhow::anyhow!("Steam virtualgamepadinfo.txt not found"));
        }
        // Is needed for steamHandles to be created
        std::env::set_var(
            "SteamVirtualGamepadInfo",
            gamepad_info_path.to_string_lossy().to_string(),
        );
    }
    Ok(())
}

pub async fn open_controller_config(app_id: u32) {
    if cef_debug::ensure::check_enabled().await {
        if cef_debug::inject::inject(
            "SharedJSContext",
            format!("SteamClient.Input.OpenDesktopConfigurator({});", app_id).as_str(),
        )
        .await
        .is_ok()
        {
            return;
        }
        tracing::warn!(
            "Failed to open Steam Input Configurator via CEF injection, falling back to steam:// URL"
        );
    }

    let steam_url = format!("steam://controllerconfig/{}", app_id);
    _ = open_steam_url(&steam_url).map_err(|e| {
        tracing::error!(
            "Failed to open Steam Input Configurator via URL {}: {}",
            steam_url,
            e
        )
    });
}
