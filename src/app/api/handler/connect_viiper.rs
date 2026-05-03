use std::{net::ToSocketAddrs, process::Child, sync::RwLock};

use axum::{Json, extract::State, response::IntoResponse};
use problem_details::ProblemDetails;
use reqwest::StatusCode;
use viiper_client::AsyncViiperClient;

use crate::{
    app::{
        api::AppState,
        input::{event::handler_events::InputHandlerEvent, sdl_loop},
    },
    config::get_config,
};

pub static SPAWNED_VIIPER: RwLock<Option<Child>> = RwLock::new(None);

/// Connect Viiper
///
/// Connects to Viiper service
#[utoipa::path(
    post,
    path = "/api/v1/connect_viiper",
    tag = "viiper",
    responses(
        (status = 200),
        (status = 401, description = "VIIPER auth required"),
        (status = 409, description = "Issue with VIIPER connection"),
        (status = 500, description = "Unknown error"),
        (status = 502, description = "Bad gateway - VIIPER not reachable"),
    )
)]
pub async fn connect_viiper(State(_state): State<AppState>) -> impl IntoResponse {
    tracing::debug!("Received request to connect to Viiper service");

    let viiper_address = get_config()
        .viiper_address
        .and_then(|s| s.to_socket_addrs().ok().and_then(|mut a| a.next()))
        .unwrap_or_else(|| "localhost:3242".to_socket_addrs().unwrap().next().unwrap());

    let viiper_pass = get_config().viiper_password.clone();
    let client =
        AsyncViiperClient::new_with_password(viiper_address, viiper_pass.unwrap_or("".to_string()));

    #[cfg(not(target_os = "linux"))]
    let mut spawn_attempted = false;

    for _ in 0..2 {
        match client.ping().await {
            Ok(resp) => {
                let is_viiper = resp.server == "VIIPER";
                if !is_viiper {
                    let msg = format!(
                        "A non-VIIPER server is running at {viiper_address} (server={}).\n\nSISR requires VIIPER to function and will now exit.",
                        resp.server
                    );
                    tracing::error!("{}", msg.replace('\n', " | "));
                    return ProblemDetails::from_status_code(StatusCode::BAD_GATEWAY)
                        .with_detail("Invalid VIIPER server")
                        .into_response();
                }
                let version = resp.version.clone();

                let min = crate::viiper_metadata::VIIPER_MIN_VERSION;
                let allow_dev = crate::viiper_metadata::VIIPER_ALLOW_DEV;
                let dev_allowed = allow_dev && (version.contains("-g") || version.contains("-dev"));
                let semver_ok = (!dev_allowed)
                    .then(|| {
                        let sv = {
                            let s = version.trim();
                            let s = s.strip_prefix('v').unwrap_or(s);
                            let prefix = s.split('-').next().unwrap_or(s);
                            let mut it = prefix.split('.');
                            let major = it.next()?.parse::<u64>().ok()?;
                            let minor = it.next().unwrap_or("0").parse::<u64>().ok()?;
                            let patch = it.next().unwrap_or("0").parse::<u64>().ok()?;
                            Some((major, minor, patch))
                        }?;

                        let mv = {
                            let s = min.trim();
                            let s = s.strip_prefix('v').unwrap_or(s);
                            let prefix = s.split('-').next().unwrap_or(s);
                            let mut it = prefix.split('.');
                            let major = it.next()?.parse::<u64>().ok()?;
                            let minor = it.next().unwrap_or("0").parse::<u64>().ok()?;
                            let patch = it.next().unwrap_or("0").parse::<u64>().ok()?;
                            Some((major, minor, patch))
                        }?;

                        Some(sv >= mv)
                    })
                    .flatten()
                    .unwrap_or(false);
                let ok = dev_allowed || semver_ok;

                if !ok {
                    let msg = format!(
                        "VIIPER is too old.\n\nDetected: {version}\nRequired: {}\n\nSISR will now exit.",
                        crate::viiper_metadata::VIIPER_MIN_VERSION
                    );
                    tracing::error!("{}", msg.replace('\n', " | "));
                    return ProblemDetails::from_status_code(StatusCode::CONFLICT)
                        .with_detail("VIIPER version not supported")
                        .into_response();
                }

                tracing::info!("VIIPER is ready (version={})", version);
                tracing::trace!("Notifying SDL input handler of VIIPER readiness");

                if let Err(e) =
                    sdl_loop::get_event_sender().push_custom_event(InputHandlerEvent::ViiperReady {
                        version: version.clone(),
                    })
                {
                    tracing::error!(
                        "Failed to notify SDL input handler of VIIPER readiness: {}",
                        e
                    );
                }
                return (StatusCode::OK, Json(serde_json::json!({}))).into_response();
            }
            Err(e) => {
                tracing::warn!("VIIPER ping failed: {}", e);

                if let viiper_client::ViiperError::Protocol(je) = e
                    && ((je.status == 401) || (je.status == 403))
                {
                    return ProblemDetails::from_status_code(StatusCode::UNAUTHORIZED)
                        .with_detail("VIIPER authentication required")
                        .into_response();
                }

                #[cfg(not(target_os = "linux"))]
                if viiper_address.ip().is_loopback() && !spawn_attempted {
                    spawn_attempted = true;

                    let spawn_res: anyhow::Result<()> = (|| {
                        use std::{path::PathBuf, process::Command};

                        let mut child_opt = SPAWNED_VIIPER
                            .write()
                            .expect("Failed to acquire spawned VIIPER lock");
                        if child_opt.is_some() {
                            return Ok(());
                        }

                        let exe_dir = std::env::current_exe()
                            .ok()
                            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
                            .unwrap_or_else(|| PathBuf::from("."));
                        let viiper_path = exe_dir.join(if cfg!(windows) {
                            "viiper.exe"
                        } else {
                            "viiper"
                        });
                        if !viiper_path.exists() {
                            anyhow::bail!(
                                "VIIPER executable not found at {}\nExpected it next to SISR.",
                                viiper_path.display()
                            );
                        }

                        let log_path = directories::ProjectDirs::from("", "", "SISR")
                            .map(|proj_dirs| proj_dirs.data_dir().join("VIIPER.log"));
                        tracing::info!("Starting local VIIPER: {}", viiper_path.display());

                        let mut cmd = Command::new(&viiper_path);
                        cmd.arg("server");
                        if let Some(log_path) = &log_path {
                            cmd.arg("--log.file").arg(log_path);
                        }
                        cmd.stdin(std::process::Stdio::null())
                            .stdout(std::process::Stdio::null())
                            .stderr(std::process::Stdio::null());

                        let child = cmd.spawn().inspect_err(|e| {
                            tracing::error!("VIIPER spawn failed: {}", e);
                        })?;
                        tracing::info!("Spawned VIIPER pid={}", child.id());
                        *child_opt = Some(child);

                        Ok(())
                    })();

                    if let Err(spawn_err) = spawn_res {
                        let msg = format!("Failed to start VIIPER locally.\n\n{spawn_err}\n");
                        tracing::error!("{}", msg.replace('\n', " | "));
                        return ProblemDetails::from_status_code(StatusCode::CONFLICT)
                            .with_detail("Failed to start VIIPER")
                            .into_response();
                    }
                }
            }
        }
    }

    ProblemDetails::from_status_code(StatusCode::CONFLICT)
        .with_detail(format!(
            "Unable to connect to VIIPER at {viiper_address} after multiple attempts."
        ))
        .into_response()
}
