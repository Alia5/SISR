use crate::app::steam;
use serde::Deserialize;

use std::{io, path::Path, process::Command, time::Duration};

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

pub fn debug_enable_file_present() -> bool {
    let Some(steam_path) = steam::util::steam_path() else {
        return false;
    };
    steam_path.join(".cef-enable-remote-debugging").exists()
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
    if debug_enable_file_present() {
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
