use axum::{Json, extract::State, response::IntoResponse};
use reqwest::StatusCode;

use crate::app::{api::AppState, updater};


/// Get Update State
///
/// Returns the current update availability and dismissal state
#[utoipa::path(
    get,
    path = "/api/v1/update",
    tag = "ui",
    responses(
        (status = 200, body = updater::UpdateStateResponse),
        (status = 500, description = "Unknown error"),
    )
)]
pub async fn get_update(
    State(_state): State<AppState>,
) -> impl IntoResponse {
    (StatusCode::OK, Json(updater::get_update_state())).into_response()
}


/// Install Update
///
/// Runs the install script for the available update
#[utoipa::path(
    post,
    path = "/api/v1/update/install",
    tag = "ui",
    responses(
        (status = 200),
        (status = 500, description = "Unknown error"),
    )
)]
pub async fn install_update(
    State(_state): State<AppState>,
) -> impl IntoResponse {
    updater::run_install_script();
    (StatusCode::OK, Json(serde_json::json!({}))).into_response()
}


/// Remind Later
///
/// Dismisses the update notification in-memory until next restart
#[utoipa::path(
    post,
    path = "/api/v1/update/remind-later",
    tag = "ui",
    responses(
        (status = 200),
        (status = 500, description = "Unknown error"),
    )
)]
pub async fn remind_later(
    State(_state): State<AppState>,
) -> impl IntoResponse {
    updater::set_remind_later();
    (StatusCode::OK, Json(serde_json::json!({}))).into_response()
}


/// Skip Version
///
/// Persistently skips the current available version
#[utoipa::path(
    post,
    path = "/api/v1/update/skip",
    tag = "ui",
    responses(
        (status = 200),
        (status = 500, description = "Unknown error"),
    )
)]
pub async fn skip_version(
    State(_state): State<AppState>,
) -> impl IntoResponse {
    updater::write_skip();
    (StatusCode::OK, Json(serde_json::json!({}))).into_response()
}


/// View on GitHub
///
/// Opens the release page in the default browser
#[utoipa::path(
    post,
    path = "/api/v1/update/view-on-github",
    tag = "ui",
    responses(
        (status = 200),
        (status = 500, description = "Unknown error"),
    )
)]
pub async fn view_on_github(
    State(_state): State<AppState>,
) -> impl IntoResponse {
    updater::open_in_browser();
    (StatusCode::OK, Json(serde_json::json!({}))).into_response()
}
