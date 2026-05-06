use axum::{Json, extract::State, response::IntoResponse};
use problem_details::ProblemDetails;
use reqwest::StatusCode;

use crate::app::{
    api::{AppState, handler},
    steam,
};

#[derive(serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct EnableCefRemoteDebugPayload {
    #[serde(default = "default_true")]
    pub restart_sisr: bool,
}

fn default_true() -> bool {
    true
}

/// Enable CEF Remote Debugging
///
/// Enables the CEF remote debugging interface of Steam by creating the required file (and restarting Steam)
#[utoipa::path(
    post,
    path = "/api/v1/enable_cef_remote_debug",
    tag = "steam",
    request_body(content = inline(EnableCefRemoteDebugPayload)),
    responses(
        (status = 200),
        (status = 500, description = "Unknown error"),
    )
)]
pub async fn enable_cef_remote_debug(
    State(_state): State<AppState>,
    body: Option<Json<EnableCefRemoteDebugPayload>>,
) -> impl IntoResponse {
    let restart_sisr_after = body.map(|b| b.restart_sisr).unwrap_or(true);
    tracing::debug!("Received request to enable CEF remote debugging (restart_sisr={restart_sisr_after})");

    if let Err(e) = steam::cef_inject::util::enable_cef_remote_debug() {
        tracing::error!("Failed to enable CEF remote debugging: {}", e);
        return ProblemDetails::from_status_code(StatusCode::INTERNAL_SERVER_ERROR)
            .with_detail(format!("{}", e))
            .into_response();
    }
    steam::util::mark_initial_setup_done();

    handler::restart_steam::do_restart_steam(restart_sisr_after)
        .await
        .into_response()
}
