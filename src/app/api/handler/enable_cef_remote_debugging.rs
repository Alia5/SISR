use axum::{extract::State, response::IntoResponse};
use problem_details::ProblemDetails;
use reqwest::StatusCode;

use crate::app::{
    api::{AppState, handler},
    steam,
};


/// Enable CEF Remote Debugging
///
/// Enables the CEF remote debugging interface of Steam by creating the required file (and restarting Steam)
#[utoipa::path(
    post,
    path = "/api/v1/enable_cef_remote_debug",
    tag = "steam",
    responses(
        (status = 200),
        (status = 500, description = "Unknown error"),
    )
)]
pub async fn enable_cef_remote_debug(
    State(_state): State<AppState>,
) -> impl IntoResponse {
    tracing::debug!("Received request to enable CEF remote debugging");

    if let Err(e) = steam::cef_inject::util::enable_cef_remote_debug() {
        tracing::error!("Failed to enable CEF remote debugging: {}", e);
        return ProblemDetails::from_status_code(StatusCode::INTERNAL_SERVER_ERROR)
            .with_detail(format!("{}", e))
            .into_response();
    }

    handler::restart_steam::restart_steam(State(_state))
        .await
        .into_response()
}
