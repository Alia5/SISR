use std::{env, ffi::OsString, process::Command};

use axum::{Json, extract::State, response::IntoResponse};
use problem_details::ProblemDetails;
use reqwest::StatusCode;

use crate::{
    app::{
        api::AppState, hid_hooks, runner::AppRunner, steam::{self}
    },
    config::get_config,
};

#[derive(serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct RestartSteamPayload {
    #[serde(default = "default_true")]
    pub restart_sisr: bool,
}

fn default_true() -> bool {
    true
}

/// Restart Steam
///
/// Attempts to restart Steam
#[utoipa::path(
    post,
    path = "/api/v1/restart_steam",
    tag = "steam",
    request_body(content = inline(RestartSteamPayload)),
    responses(
        (status = 200),
        (status = 500, description = "Unknown error"),
    )
)]
pub async fn restart_steam(
    State(_state): State<AppState>,
    body: Option<Json<RestartSteamPayload>>,
) -> impl IntoResponse {
    let restart_sisr_after = body.map(|b| b.restart_sisr).unwrap_or(true);
    tracing::debug!("Received request to restart Steam (restart_sisr={restart_sisr_after})");
    do_restart_steam(restart_sisr_after).await
}

pub async fn do_restart_steam(restart_sisr_after: bool) -> impl IntoResponse {

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
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

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

    if restart_sisr_after {
        tracing::debug!("Scheduling SISR self-restart after Steam restart");
        let Ok(current_exe) = env::current_exe() else {
            tracing::error!("Failed to resolve current executable for SISR restart");
            return (StatusCode::OK, Json(serde_json::json!({}))).into_response();
        };
        let current_pid = std::process::id();
        let current_args: Vec<OsString> = env::args_os().skip(1).collect();

        #[cfg(windows)]
        {
            let ps_quote = |value: &str| format!("'{}'", value.replace('\'', "''"));
            let exe = ps_quote(&current_exe.as_os_str().to_string_lossy());
            let arg_list = if current_args.is_empty() {
                String::from("@()")
            } else {
                format!(
                    "@({})",
                    current_args
                        .iter()
                        .map(|arg| ps_quote(&arg.to_string_lossy()))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            };
            let command = format!(
                "& {{ while (Get-Process -Id {current_pid} -ErrorAction SilentlyContinue) {{ Start-Sleep -Milliseconds 200 }}; Start-Process -FilePath {exe} -ArgumentList {arg_list} }}"
            );
            if let Err(e) = Command::new("powershell")
                .args(["-NoProfile", "-NonInteractive", "-WindowStyle", "Hidden", "-Command", &command])
                .spawn()
            {
                tracing::error!("Failed to schedule SISR restart: {}", e);
            }
        }
        #[cfg(target_os = "linux")]
        {
            let pid = current_pid.to_string();
            if let Err(e) = Command::new("sh")
                .arg("-lc")
                .arg("while kill -0 \"$1\" 2>/dev/null; do sleep 0.2; done; shift; exec \"$@\"")
                .arg("restart_sisr")
                .arg(&pid)
                .arg(&current_exe)
                .args(&current_args)
                .spawn()
            {
                tracing::error!("Failed to schedule SISR restart: {}", e);
            }
        }

        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(150)).await;
            AppRunner::shutdown();
        });
    }

    (StatusCode::OK, Json(serde_json::json!({}))).into_response()
}
