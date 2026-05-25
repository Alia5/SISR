use crate::app::steam;
use serde::Deserialize;

use std::{
    io,
    path::Path,
    process::Command,
    time::Duration,
};

pub const DEFAULT_CEF_DEBUG_PORT: u16 = 8080;

#[derive(Deserialize)]
pub struct TabInfo {
    #[serde(rename = "description")]
    pub description: String,
    #[serde(rename = "devtoolsFrontendUrl")]
    pub devtools_frontend_url: String,
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "title")]
    pub title: String,
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(rename = "url")]
    pub url: String,
    #[serde(rename = "webSocketDebuggerUrl")]
    pub websocket_debugger_url: String,
}

pub fn cef_debugging_enabled() -> bool {
    let Some(steam_path) = steam::util::steam_path() else {
        return false;
    };
    if steam_path.join(".cef-enable-remote-debugging").exists() {
        return true;
    }
    cef_remote_debug_port() != DEFAULT_CEF_DEBUG_PORT
}

pub fn cef_remote_debug_port() -> u16 {
    let default_port = DEFAULT_CEF_DEBUG_PORT;

    #[cfg(target_os = "linux")]
    {
        return detect_cef_remote_debug_port_linux(default_port);
    }

    #[cfg(target_os = "windows")]
    {
        return detect_cef_remote_debug_port_windows(default_port);
    }

    #[allow(unreachable_code)]
    default_port
}

pub async fn get_cef_tabs(port: u16) -> anyhow::Result<Vec<TabInfo>> {
    if port == 0 {
        anyhow::bail!("CEF remote debug port not configured");
    }

    let url = format!("http://localhost:{port}/json");
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()?;
    let response = client.get(url).send().await?;

    if !response.status().is_success() {
        anyhow::bail!(
            "unexpected status code from CEF remote debug endpoint: {}",
            response.status()
        );
    }

    Ok(response.json::<Vec<TabInfo>>().await?)
}

pub async fn cef_remote_debug_reachable(port: u16) -> bool {
    let Ok(tabs) = get_cef_tabs(port).await else {
        return false;
    };

    tabs.iter()
        .any(|tab| tab.url.starts_with("https://steamloopback.host"))
}

pub fn enable_cef_remote_debug() -> anyhow::Result<bool> {
    if cef_debugging_enabled() {
        return Ok(false);
    }

    let Some(steam_dir) = steam::util::steam_path() else {
        anyhow::bail!("Steam path not found");
    };

    let file_path = steam_dir.join(".cef-enable-remote-debugging");
    if let Err(err) = std::fs::File::create(&file_path) {
        tracing::error!(error = %err, "Failed to create CEF remote debug enable file");
        if err.kind() != io::ErrorKind::PermissionDenied {
            return Err(err.into());
        }

        tracing::error!(error = %err, "direct file creation failed, attempting elevated creation");
        create_cef_file_elevated(&file_path)?;
    }

    Ok(true)
}

pub fn create_cef_file_elevated(file_path: &Path) -> anyhow::Result<()> {
    #[cfg(not(target_os = "windows"))]
    {
        anyhow::bail!(
            "elevated file creation not supported on {}",
            std::env::consts::OS
        );
    }

    #[cfg(target_os = "windows")]
    {
        let escaped_file_path = file_path.to_string_lossy().replace('\'', "''");
        let elevated_command = format!(
            "New-Item -ItemType File -Path '{}' -Force | Out-Null",
            escaped_file_path
        );
        let command = format!(
            "Start-Process 'powershell' -Verb RunAs -Wait -ArgumentList '-ExecutionPolicy','Bypass','-Command','{}'",
            elevated_command
        );

        let status = Command::new("powershell")
            .args(["-ExecutionPolicy", "Bypass", "-Command", &command])
            .status()?;

        if !status.success() {
            anyhow::bail!("powershell exited with status: {}", status);
        }

        if !file_path.exists() {
            anyhow::bail!(
                "elevated PowerShell command completed but file was not created: {}",
                file_path.display()
            );
        }

        Ok(())
    }
}

#[cfg(target_os = "linux")]
fn detect_cef_remote_debug_port_linux(default_port: u16) -> u16 {
    const PREFIXES: [&str; 2] = [
        "--remote-debugging-port=",
        "--remote-debugging-io-pipes=",
    ];

    let Ok(entries) = std::fs::read_dir("/proc") else {
        return default_port;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let Some(pid) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };

        if !pid.bytes().all(|ch| ch.is_ascii_digit()) {
            continue;
        }

        let Ok(comm) = std::fs::read_to_string(path.join("comm")) else {
            continue;
        };

        if !comm.trim().eq_ignore_ascii_case("steamwebhelper") {
            continue;
        }

        let Ok(cmdline) = std::fs::read(path.join("cmdline")) else {
            continue;
        };

        if cmdline.is_empty() {
            continue;
        }

        let cmdline = String::from_utf8_lossy(&cmdline).replace('\0', " ");
        for arg in cmdline.split_whitespace() {
            if let Some(port) = parse_remote_debug_port_arg(arg, &PREFIXES) {
                return port;
            }
        }
    }

    default_port
}

#[cfg(target_os = "windows")]
fn detect_cef_remote_debug_port_windows(default_port: u16) -> u16 {
    const PREFIXES: [&str; 2] = [
        "--remote-debugging-port=", //Millennium v2
        "--remote-debugging-io-pipes=", //Millennium v3 (Beta)
    ];

    use sysinfo::{ProcessesToUpdate, System};

    let mut system = System::new_all();
    system.refresh_processes(ProcessesToUpdate::All, true);

    for process in system.processes().values() {
        let name = process.name().to_string_lossy();
        if !name.eq_ignore_ascii_case("steamwebhelper.exe")
            && !name.eq_ignore_ascii_case("steamwebhelper")
        {
            continue;
        }

        for arg in process.cmd() {
            let arg = arg.to_string_lossy();
            if let Some(port) = parse_remote_debug_port_arg(arg.trim_matches('"'), &PREFIXES) {
                return port;
            }
        }
    }

    default_port
}

fn parse_remote_debug_port_arg(arg: &str, prefixes: &[&str]) -> Option<u16> {

    for prefix in prefixes {
        if let Some(port_str) = arg.strip_prefix(prefix) {
            let first = port_str.split(',').next().unwrap_or("").trim(); // Millennium devs give two ports seperated by a comma but only the first one is needed - Josh
            let digits: String = first.chars().filter(|c| c.is_ascii_digit()).collect();
            if !digits.is_empty() {
                if let Ok(port) = digits.parse::<u16>() {
                    if port != 0 {
                        return Some(port);
                    }
                }
            }

            return Some(DEFAULT_CEF_DEBUG_PORT); // for some reason this fixes everything for millennium detection, so here it is. Does not affect non-millennium clients - Josh
        }
    }

    None
}
