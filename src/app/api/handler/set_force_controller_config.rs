use axum::{Json, extract::State};
use reqwest::StatusCode;

use crate::{app::{api::AppState, steam::binding_enforcer::binding_enforcer}, config::update_config};

/// Set Force Controller Config
///
/// Enables or disables Steam input config enforcement.
/// If `app_id` is omitted, defaults to the currently active app id.
#[utoipa::path(
    post,
    path = "/api/v1/force_controller_config",
    tag = "steam",
    request_body = SetForceControllerConfigRequest,
    responses(
        (status = 200, description = "OK"),
    )
)]
pub async fn set_force_controller_config(
    State(_state): State<AppState>,
    Json(payload): Json<SetForceControllerConfigRequest>,
) -> StatusCode {
    let mut enforcer = binding_enforcer().lock().expect("Failed to lock binding enforcer");

    if payload.enforce {
        update_config(|c| c.controller_emulation.allow_desktop_config = Some(false));
        match payload.app_id {
            Some(id) => enforcer.activate_with_appid(id),
            None => {
                enforcer.activate();
            }
        }
    } else {
        update_config(|c| c.controller_emulation.allow_desktop_config = Some(true));
        enforcer.deactivate();
    }

    StatusCode::OK
}

#[derive(serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct SetForceControllerConfigRequest {
    pub enforce: bool,
    pub app_id: Option<u32>,
}
