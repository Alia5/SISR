
use axum::{ Json};
use problem_details::ProblemDetails;
use reqwest::StatusCode;
use serde::Serialize;
use utoipa::ToSchema;

use crate::app::steam::{self, cef_inject};


/// Get Steam CEF Remote Debugging Reachability
///
/// Returns whether the CEF remote debugging interface of Steam is reachable.
#[utoipa::path(
    get,
    path = "/api/v1/steam_cef_reachable",
    tag = "steam",
    responses(
        (status = 200, body = RemoteCefReachable),
        (status = 500, body = ProblemDetails)
    )
)]
pub async fn steam_cef_reachable() -> (StatusCode, Json<RemoteCefReachable>) {

    tracing::debug!("Received request for Steam CEF remote debugging reachability");
   
    let steam_running = steam::util::steam_running();
    let cef_enable_file_present = steam::cef_inject::util::debug_enable_file_present();

    let cef_reachable = if steam_running && cef_enable_file_present {
        cef_inject::util::cef_remote_debug_reachable(8080).await
    } else {
        false
    };

    (
        StatusCode::OK,
        Json(RemoteCefReachable {
            reachable: cef_reachable,
        }),
    )
}

#[derive(Serialize, ToSchema)]
pub struct RemoteCefReachable {
    pub reachable: bool,
}