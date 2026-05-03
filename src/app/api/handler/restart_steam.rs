use axum::{Json, extract::State, response::IntoResponse};
use problem_details::ProblemDetails;
use reqwest::StatusCode;

use crate::{
    app::{
        api::AppState, hid_hooks, steam::{self}
    },
    config::get_config,
};

/// Restart Steam
///
/// Attempts to restart Steam
#[utoipa::path(
    post,
    path = "/api/v1/restart_steam",
    tag = "steam",
    responses(
        (status = 200),
        (status = 500, description = "Unknown error"),
    )
)]
pub async fn restart_steam(State(_state): State<AppState>) -> impl IntoResponse {
    tracing::debug!("Received request to restart Steam");

    if steam::util::steam_running() {
        let _ = steam::util::open_url("steam://exit");

        for _ in 0..10 {
            if !steam::util::steam_running() {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }

    }
    steam::util::open_url("steam://open/main")
        .map_err(|e| {
            tracing::error!("Failed to restart Steam: {}", e);
            ProblemDetails::from_status_code(StatusCode::INTERNAL_SERVER_ERROR)
                .with_detail(format!("{}", e))
        })
        .ok();
    for _ in 0..19990 {
        let steam_running = steam::util::steam_running();
        let active_user = steam::util::active_user_id();
        if steam_running && active_user.is_some_and(|id| id != 0) {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    if !steam::util::launched_via_steam() && !get_config().steam.no_steam.unwrap_or(false) {
		#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
		{
			tracing::debug!("Uninstalling HID detours before unloading Steam overlay");
			hid_hooks::rehook::unhook_all();
		}

        steam::util::unload_steam_overlay();
        // HACK!
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        hid_hooks::hid_check::enumerate_hid_exports();

        match steam::util::try_set_marker_steam_env() {
            Ok(_) => {
                tracing::info!("Successfully set marker Steam environment variables");
                steam::util::load_steam_overlay();
            }
            Err(e) => {
                tracing::error!("Failed to set marker Steam environment variables: {}", e);
                // TODO: some error handling, whatever
            }
        }
        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        {
            let hooked_by_steam = hid_hooks::hid_check::detect_hid_hooks();

            if let Some(baselines) = hid_hooks::hid_check::EXPORTS_BASELINE.get() {
                for (name, bytes) in baselines {
                    let mut hex = String::from("0x");
                    for b in *bytes {
                        hex.push_str(&format!("{:02x}", b));
                    }
                    tracing::trace!("Baseline bytes: {}: \"{}\"", name, hex);
                }
            }

            for hook in &hooked_by_steam {
                tracing::info!("Detected HID hook by Steam: {}", hook);
                hid_hooks::rehook::rehook(hook);
            }
        }
    }

    (StatusCode::OK, Json(serde_json::json!({}))).into_response()
}
