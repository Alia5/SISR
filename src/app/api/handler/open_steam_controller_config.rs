use axum::Json;
use axum::http::StatusCode;

use crate::app::steam::util::open_controller_config;

/// Open Steam Controller Config
///
/// Opens the Steam controller configuration overlay for the given app id.
/// If `app_id` is omitted, defaults to the currently active app id (`SteamAppId` env var).
#[utoipa::path(
    post,
    path = "/api/v1/open_steam_controller_config",
    tag = "steam",
    request_body(content = Option<OpenSteamControllerConfigRequest>),
    responses(
        (status = 200, description = "OK"),
        (status = 400, description = "No app id available"),
    )
)]
pub async fn open_steam_controller_config(
    payload: Option<Json<OpenSteamControllerConfigRequest>>,
) -> StatusCode {
    let app_id = payload
        .and_then(|p| p.app_id)
        .or_else(|| {
            std::env::var("SteamAppId")
                .ok()
                .and_then(|v| v.parse::<u32>().ok())
                .filter(|&id| id != 0)
        });

    let Some(app_id) = app_id else {
        tracing::warn!("open_steam_controller_config: no app id available");
        return StatusCode::BAD_REQUEST;
    };

    open_controller_config(app_id).await;
    StatusCode::OK
}

#[derive(serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct OpenSteamControllerConfigRequest {
    pub app_id: Option<u32>,
}
