use axum::{Json, extract::State, response::IntoResponse};
use reqwest::StatusCode;

use crate::app::{api::AppState, window::event::{WindowRunnerEvent, get_event_sender}};


/// Change UI State
///
/// Changes the UI state (show/hide)
#[utoipa::path(
    post,
    path = "/api/v1/show_hide_ui",
    tag = "ui",
    request_body = ChangeUiStatePayload,
    responses(
        (status = 200),
        (status = 500, description = "Unknown error"),
    )
)]
pub async fn change_ui_state(
    State(_state): State<AppState>,
    body: Json<ChangeUiStatePayload>
) -> impl IntoResponse {
    tracing::debug!("Received request to change UI state: {:?}", body.show);

    if let Err(e) = get_event_sender().send_event(WindowRunnerEvent::ToggleUi(body.show)) {
        tracing::error!("Failed to send ToggleUi event: {:?}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Failed to change UI state"}))).into_response();
    }

    (StatusCode::OK, Json(serde_json::json!({}))).into_response()
}


#[derive(serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct ChangeUiStatePayload {
    pub show: bool,
}