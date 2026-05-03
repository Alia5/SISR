use axum::{Json, extract::State, response::IntoResponse};
use problem_details::ProblemDetails;
use reqwest::StatusCode;

use crate::app::{api::AppState, steam};

const OVERLAY_NOTIFIER_SCRIPT: &str = include_str!("../../../../CEF_Payloads/dist/overlay_callback.js");


/// Inject Overlay Notifier
///
/// Injects overlay notifier into Steams shared context
#[utoipa::path(
    post,
    path = "/api/v1/inject_overlay_notifier",
    tag = "inject",
    responses(
        (status = 200),
        (status = 409, description = "Steam CEF Debugging not reachable"),
        (status = 500, description = "Unknown error"),
    )
)]
pub async fn inject_overlay_notifier(
    State(_state): State<AppState>
) -> impl IntoResponse {
    tracing::debug!("Received request to inject overlay notifier");

    let file_present = steam::cef_inject::util::debug_enable_file_present();
    if !file_present {
        return ProblemDetails::from_status_code(StatusCode::CONFLICT)
            .with_detail(
                "Steam CEF remote debugging is not enabled."
                    .to_string(),
            )
            .into_response();
    }
    let reachable = steam::cef_inject::util::cef_remote_debug_reachable(8080).await;
    if !reachable {
        return ProblemDetails::from_status_code(StatusCode::CONFLICT)
            .with_detail(
                "Steam CEF remote debugging is not reachable.".to_string(),
            )
            .into_response();
    }

    if let Err(e) = steam::cef_inject::injector::inject::<()>(OVERLAY_NOTIFIER_SCRIPT).await {
        tracing::error!("Failed to inject overlay notifier: {}", e);
        return ProblemDetails::from_status_code(StatusCode::INTERNAL_SERVER_ERROR)
            .with_detail(format!("{}", e))
            .into_response();
    }

    (StatusCode::OK, Json(serde_json::json!({}))).into_response()
}
