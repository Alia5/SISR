use crate::config::{UpdateNotify, get_config};

use std::sync::Mutex;

use tracing::{error, info, warn};

static REMIND_LATER_VERSION: Mutex<Option<String>> = Mutex::new(None);

struct UpdateInfo {
    version: String,
    html_url: String,
    install_channel: String,
}

static AVAILABLE_UPDATE: Mutex<Option<UpdateInfo>> = Mutex::new(None);

macro_rules! current_version {
    () => {
        match option_env!("SISR_VERSION") {
            Some(v) => v,
            None => env!("CARGO_PKG_VERSION"),
        }
    };
}
const CURRENT_VERSION: &str = current_version!();

#[derive(Debug)]
struct Version {
    major: u64,
    minor: u64,
    patch: u64,
    commits: u64,
}

fn parse_version(s: &str) -> Option<Version> {
    let s = s.trim().strip_prefix('v').unwrap_or(s);

    let mut main_and_rest = s.splitn(2, '-');
    let main_part = main_and_rest.next()?;

    let commits_part = main_and_rest.next().and_then(|rest| {
        let first_segment = rest.split('-').next()?;
        first_segment.parse::<u64>().ok()
    });

    let mut parts = main_part.split('.');
    let major = parts.next()?.parse::<u64>().ok()?;
    let minor = parts.next().unwrap_or("0").parse::<u64>().ok()?;
    let patch = parts.next().unwrap_or("0").parse::<u64>().ok()?;

    Some(Version {
        major,
        minor,
        patch,
        commits: commits_part.unwrap_or(0),
    })
}

impl Version {
    fn is_newer_than(&self, other: &Version) -> bool {
        (self.major, self.minor, self.patch, self.commits)
            > (other.major, other.minor, other.patch, other.commits)
    }
}

fn dismissed_file_path() -> Option<std::path::PathBuf> {
    directories::ProjectDirs::from("", "", "SISR")
        .map(|proj| proj.data_dir().join("update-dismissed"))
}

fn is_dismissed(ver: &str) -> bool {
    let Some(path) = dismissed_file_path() else {
        return false;
    };
    std::fs::read_to_string(&path)
        .map(|content| content.trim() == ver)
        .unwrap_or(false)
}

pub fn clear_dismissed() {
    if let Some(path) = dismissed_file_path() {
        let _ = std::fs::remove_file(path);
    }
}

fn write_dismissed(ver: &str) {
    let Some(path) = dismissed_file_path() else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Err(e) = std::fs::write(&path, ver) {
        error!("Failed to write update-dismissed file: {}", e);
    }
}

fn is_remind_later(ver: &str) -> bool {
    REMIND_LATER_VERSION
        .lock()
        .ok()
        .and_then(|g| g.as_deref().map(|v| v == ver))
        .unwrap_or(false)
}

pub fn set_remind_later() {
    let guard = AVAILABLE_UPDATE.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(info) = guard.as_ref() {
        if let Ok(mut g) = REMIND_LATER_VERSION.lock() {
            *g = Some(info.version.clone());
        }
    }
}

pub fn clear_remind_later() {
    if let Ok(mut g) = REMIND_LATER_VERSION.lock() {
        *g = None;
    }
}

pub fn write_skip() {
    let guard = AVAILABLE_UPDATE.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(info) = guard.as_ref() {
        write_dismissed(&info.version.clone());
    }
}

#[derive(serde::Deserialize)]
struct GithubRelease {
    tag_name: String,
    name: Option<String>,
    prerelease: bool,
    html_url: String,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct UpdateStateResponse {
    pub update_available: bool,
    pub update_version: Option<String>,
    /// In-memory only: cleared on restart (remind later)
    pub dismissed: bool,
    /// Persisted: skip this version
    pub skipped: bool,
}

pub fn get_update_state() -> UpdateStateResponse {
    let guard = AVAILABLE_UPDATE.lock().unwrap_or_else(|e| e.into_inner());
    match guard.as_ref() {
        None => UpdateStateResponse {
            update_available: false,
            update_version: None,
            dismissed: false,
            skipped: false,
        },
        Some(info) => UpdateStateResponse {
            update_available: true,
            update_version: Some(info.version.clone()),
            dismissed: is_remind_later(&info.version),
            skipped: is_dismissed(&info.version),
        },
    }
}

pub fn run_install_script() {
    let channel = AVAILABLE_UPDATE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .as_ref()
        .map(|i| i.install_channel.clone())
        .unwrap_or_else(|| "stable".to_string());
    run_install_script_for_channel(&channel);
}

pub fn open_in_browser() {
    let url = AVAILABLE_UPDATE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .as_ref()
        .map(|i| i.html_url.clone());
    if let Some(url) = url {
        let result = if cfg!(target_os = "windows") {
            std::process::Command::new("cmd").args(["/c", "start", &url]).spawn()
        } else {
            std::process::Command::new("xdg-open").arg(&url).spawn()
        };
        if let Err(e) = result {
            error!("Failed to open browser: {}", e);
        }
    }
}

fn run_install_script_for_channel(channel: &str) {
    let base_url = format!("https://alia5.github.io/SISR/{}/install", channel);

    if cfg!(target_os = "windows") {
        let url = format!("{}.ps1", base_url);
        let result = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                &format!(
                    "Start-Process powershell -ArgumentList '-NoExit -NoProfile -ExecutionPolicy Bypass -Command \"iwr -useb {} | iex\"' -Verb RunAs",
                    url
                ),
            ])
            .spawn();
        if let Err(e) = result {
            error!("Failed to run install script: {}", e);
        }
    } else {
        let url = format!("{}.sh", base_url);
        let result = std::process::Command::new("sh")
            .args(["-c", &format!("curl -fsSL '{}' | sh", url)])
            .spawn();
        if let Err(e) = result {
            error!("Failed to run install script: {}", e);
        }
    }
}

pub fn should_notify(ver: &str) -> bool {
    !is_dismissed(ver) && !is_remind_later(ver)
}

pub async fn check() -> Option<String> {
    let notify = get_config().update_notify.unwrap_or(UpdateNotify::Stable);

    if notify == UpdateNotify::None {
        return None;
    }

    let cur = match parse_version(CURRENT_VERSION) {
        Some(v) => v,
        None => {
            if CURRENT_VERSION != "dev" {
                warn!("Failed to parse current SISR version: {}", CURRENT_VERSION);
            }
            return None;
        }
    };



    let release = match fetch_release(notify).await {
        Some(r) => r,
        None => return None,
    };

    let version_source = if release.prerelease {
        release.name.as_deref().unwrap_or(&release.tag_name)
    } else {
        &release.tag_name
    };

    let remote = match parse_version(version_source) {
        Some(v) => v,
        None => {
            warn!("Failed to parse remote version: {}", version_source);
            return None;
        }
    };

    if !remote.is_newer_than(&cur) {
        return None;
    }

    let version = version_source
        .trim()
        .strip_prefix('v')
        .map(|s| format!("v{}", s))
        .unwrap_or_else(|| format!("v{}", version_source.trim()));

    let install_channel = if notify == UpdateNotify::Prerelease {
        "main"
    } else {
        "stable"
    };

    info!(
        "SISR update available: current={}, available={}",
        CURRENT_VERSION, version
    );

    if let Ok(mut guard) = AVAILABLE_UPDATE.lock() {
        *guard = Some(UpdateInfo {
            version: version.clone(),
            html_url: release.html_url.clone(),
            install_channel: install_channel.to_string(),
        });
    }

    Some(version)
}

async fn fetch_release(notify: UpdateNotify) -> Option<GithubRelease> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("SISR-updater")
        .build()
        .ok()?;

    if notify == UpdateNotify::Prerelease {
        let resp = client
            .get("https://api.github.com/repos/Alia5/SISR/releases?per_page=1")
            .send()
            .await
            .ok()?;
        if !resp.status().is_success() {
            warn!(
                "GitHub API returned status {} when fetching releases",
                resp.status()
            );
            return None;
        }
        let releases: Vec<GithubRelease> = resp.json().await.ok()?;
        releases.into_iter().next()
    } else {
        let resp = client
            .get("https://api.github.com/repos/Alia5/SISR/releases/latest")
            .send()
            .await
            .ok()?;
        if !resp.status().is_success() {
            warn!(
                "GitHub API returned status {} when fetching latest release",
                resp.status()
            );
            return None;
        }
        resp.json().await.ok()
    }
}
