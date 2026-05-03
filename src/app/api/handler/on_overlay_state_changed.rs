use axum::Json;
use problem_details::ProblemDetails;
use reqwest::StatusCode;

use crate::app::{input::{event::handler_events::InputHandlerEvent, sdl_loop}, window::{self, event::WindowRunnerEvent}};



/// On overlay change
///
/// Callback for when the steam overlay is opened or closed. 
#[utoipa::path(
    post,
    path = "/api/v1/overlay_state_changed",
    tag = "callback",
    request_body = OverlayStateChangePayload,
    responses(
        (status = 200),
        (status = 422, body = ProblemDetails)
    )
)]
pub async fn on_overlay_state_changed(body: Json<OverlayStateChangePayload>) -> StatusCode {

    tracing::debug!("Received overlay state change: open={}", body.open);

    if let Err(e) = window::event::get_event_sender()
        .send_event(WindowRunnerEvent::OverlayStateChanged(body.open)) {
        tracing::error!("Failed to send overlay state change event: {}", e);
    }
    if let Err(e) = sdl_loop::get_event_sender()
        .push_custom_event(InputHandlerEvent::OverlayStateChanged { open: body.open }) {
        tracing::error!("Failed to send overlay state change event to input handler: {}", e);
    }

    StatusCode::OK
}

#[derive(serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct OverlayStateChangePayload {
    pub open: bool,
}