use axum::{Json, extract::State, response::IntoResponse};
use problem_details::ProblemDetails;
use reqwest::StatusCode;
use std::path::PathBuf;

use crate::app::{api::AppState, steam};

const CREATE_MARKER_SHORTCUT_SCRIPT: &str =
    include_str!("../../../../CEF_Payloads/dist/create_marker_shortcut.js");

/// Create Marker Shortcut
///
/// Creates a marker shortcut in Steam
#[utoipa::path(
    post,
    path = "/api/v1/create_marker_shortcut",
    tag = "inject",
    responses(
        (status = 200),
        (status = 409, description = "Marker Shortcut present / Steam CEF Debugging not reachable"),
        (status = 500, description = "Unknown error"),
    )
)]
pub async fn create_marker_shortcut(State(_state): State<AppState>) -> impl IntoResponse {
    tracing::debug!("Received request to inject overlay notifier");

    let Some(steam_path) = std::env::var("SteamPath")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
        .or_else(steam::util::steam_path) else {
            return ProblemDetails::from_status_code(StatusCode::CONFLICT)
                .with_detail("Steam path not found.".to_string())
                .into_response();
        };
    let Some(active_user_id) = steam::util::active_user_id() else {
        return ProblemDetails::from_status_code(StatusCode::CONFLICT)
            .with_detail("No active Steam user found.".to_string())
            .into_response();
    };

    let Some(shortcuts_path) = steam::util::get_shortcuts_path(&steam_path, active_user_id) else {
        return ProblemDetails::from_status_code(StatusCode::CONFLICT)
            .with_detail("Shortcuts path not found.".to_string())
            .into_response();
    };
    let marker_exists = steam::util::shortcuts_has_sisr_marker(&shortcuts_path);
    if marker_exists > 0{
        return (StatusCode::OK, Json(serde_json::json!({}))).into_response()
    };

    let file_present = steam::cef_inject::util::debug_enable_file_present();
    if !file_present {
        return ProblemDetails::from_status_code(StatusCode::CONFLICT)
            .with_detail("Steam CEF remote debugging is not enabled.".to_string())
            .into_response();
    }
    let reachable = steam::cef_inject::util::cef_remote_debug_reachable(8080).await;
    if !reachable {
        return ProblemDetails::from_status_code(StatusCode::CONFLICT)
            .with_detail("Steam CEF remote debugging is not reachable.".to_string())
            .into_response();
    }

    let own_executable_path = std::env::current_exe()
        .ok()
        .and_then(|path| path.into_os_string().into_string().ok())
        .unwrap_or_else(|| "unknown".to_string());

    let payload = format!(
        "var SISR_PATH = `{}`;\n{}",
        own_executable_path.replace("\\", "/"),
        CREATE_MARKER_SHORTCUT_SCRIPT
    );

    if let Err(e) = steam::cef_inject::injector::inject::<serde_json::Value>(&payload).await {
        tracing::error!("Failed to inject marker shortcut: {}", e);

        let marker_after_error = steam::util::shortcuts_has_sisr_marker(&shortcuts_path);
        if marker_after_error > 0 {
            tracing::warn!(
                "Injection reported error, but marker shortcut now exists (appid: {}); treating as success",
                marker_after_error
            );
            return (StatusCode::OK, Json(serde_json::json!({}))).into_response();
        }

        return ProblemDetails::from_status_code(StatusCode::INTERNAL_SERVER_ERROR)
            .with_detail(format!("{}", e))
            .into_response();
    }

    (StatusCode::OK, Json(serde_json::json!({}))).into_response()
}
