use axum::{Json, extract::State};
use problem_details::ProblemDetails;
use reqwest::StatusCode;
use serde::Serialize;
use utoipa::ToSchema;

use crate::{app::{api::AppState, steam}, config::get_config};

/// Get Steam Status
///
/// Returns information about Steam, the SISR marker shortcut, and the CEF remote debugging (of Steam)
#[utoipa::path(
    get,
    path = "/api/v1/steam_status",
    tag = "steam",
    responses(
        (status = 200, body = SteamStatus),
        (status = 500, body = ProblemDetails)
    )
)]

pub async fn steam_status(
    State(_state): State<AppState>
) -> (StatusCode, Json<SteamStatus>) {
    tracing::debug!("Received request for Steam status");

    let steam_path = std::env::var("SteamPath")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            steam::util::steam_path().map(|path| path.to_string_lossy().to_string())
        });
    let steam_app_id = std::env::var("SteamAppId")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(0);
    let mut marker_appid = if steam_app_id != 0 {
        steam_app_id
    } else {
        std::env::var("SISR_MARKER_ID")
            .ok()
            .and_then(|value| value.parse::<u32>().ok())
            .unwrap_or(0)
    };
    if marker_appid == 0 {
        // clipppyyyy!!!
        if let Some(steam_path) = steam_path.as_ref().map(std::path::PathBuf::from) {
            // NO!
            if let Some(active_user_id) = steam::util::active_user_id() {
                // AUS! BÖSES CLIPPY!
                if let Some(shortcuts_path) = steam::util::get_shortcuts_path(&steam_path, active_user_id) {
                    marker_appid = steam::util::shortcuts_has_sisr_marker(&shortcuts_path);
                }
            }
        }
    }

    let steam_game_id = std::env::var("SteamGameId")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|id| *id != 0)
        .or_else(|| {
            std::env::var("SteamOverlayGameId")
                .ok()
                .and_then(|value| value.parse::<u64>().ok())
                .filter(|id| *id != 0)
        })
        .unwrap_or(0);

    let steam_running = steam::util::steam_running();

    let steam_cef_port = 8080; // TODO: compatibility with stupid millenium (it changes the port via hooking into steam)

    let cef_enable_file_present = steam::cef_inject::util::debug_enable_file_present();

    (
        StatusCode::OK,
        Json(SteamStatus {
            no_steam_mode: get_config().steam.no_steam.unwrap_or(false),
            remote_debug: RemoteDebugStatus {
                enabled: cef_enable_file_present,
                port: steam_cef_port,
            },
            running: steam_running,
            marker_shortcut_present: marker_appid != 0,
            marker_app_id: if marker_appid == 0 {
                None
            } else {
                Some(marker_appid)
            },
            steam_game_id,
            path: steam_path,
            launched_via_steam: steam::util::launched_via_steam(),
        }),
    )
}

#[derive(Serialize, ToSchema)]
pub struct SteamStatus {
    pub no_steam_mode: bool,
    pub remote_debug: RemoteDebugStatus,
    pub running: bool,
    pub marker_shortcut_present: bool,
    pub marker_app_id: Option<u32>,
    pub steam_game_id: u64,
    pub path: Option<String>,
    pub launched_via_steam: bool,
}

#[derive(Serialize, ToSchema)]
pub struct RemoteDebugStatus {
    pub enabled: bool,
    pub port: u16,
}
