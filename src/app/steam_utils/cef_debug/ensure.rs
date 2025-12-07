use std::{
    process::Command,
    sync::{Arc, Mutex, OnceLock},
};

use tokio::runtime::Handle;
use tracing::{debug, error, info, trace, warn};
use winit::event_loop::EventLoopProxy;

use crate::{
    app::{
        gui::dialogs::{self, Dialog, push_dialog},
        steam_utils::util::{self, launched_via_steam, open_steam_url, steam_path},
        window::RunnerEvent,
    },
    config::CONFIG,
};

pub static CEF_DEBUG_PORT: OnceLock<u16> = OnceLock::new();

pub async fn check_enabled() -> bool {
    CEF_DEBUG_PORT.get_or_init(|| 8080);
    // http://localhost:8080/json/list <- tab list json / must contain "Steam" stuff.
    // TODO: Configurable port
    // NOTE: Steam itself does not provide a configurable port
    // TODO: create x-platform util to hook into steam to be able to change the port....
    // fuck me...

    let timeout = tokio::time::timeout(
        std::time::Duration::from_secs(1),
        reqwest::get("http://localhost:8080/json/list"),
    );

    if let Ok(Ok(response)) = timeout.await
        && let Ok(body) = response.text().await
    {
        trace!("Steam CEF Debug detected via HTTP endpoint");
        return body.contains("Steam");
    }
    trace!("Steam CEF Debug not detected via HTTP endpoint");
    false
}

pub fn check_enable_file() -> bool {
    let Some(steam_path) = util::steam_path() else {
        error!("Steam path not found");
        return false;
    };

    let debug_file_path = steam_path.join(".cef-enable-remote-debugging");
    if debug_file_path.exists() {
        trace!(
            "Steam CEF Debug enable file found: {}",
            debug_file_path.display()
        );
        return true;
    }
    trace!(
        "Steam CEF Debug enable file not found: {}",
        debug_file_path.display()
    );

    false
}

// ----

// Yup! This code will do for now!
// Believe or not, this is not LLM generated 🫣
// Just stop reading!
pub async fn ensure_cef_enabled(
    winit_waker: Arc<Mutex<Option<EventLoopProxy<RunnerEvent>>>>,
) -> (bool, bool) {
    let Some(steam_path) = steam_path() else {
        error!("Steam installation path could not be found");
        let winit_waker = winit_waker.clone();
        _ = push_dialog(Dialog::new_ok(
            "Steam not found",
            "Steam installation could not be found.
                    SISR cannot function without Steam
                    SISR will now quit",
            move || {
                let Ok(guard) = winit_waker.lock() else {
                    panic!("Failed to acquire winit waker lock to quit app after missing steam")
                };
                let Some(proxy) = &*guard else {
                    panic!("Winit waker not initialized to quit app after missing steam")
                };
                _ = proxy.send_event(RunnerEvent::Quit());
            },
        ));
        // might as well, continue without, may be started from steam, and path is broken?
        return (false, true);
    };

    // I know... whatever

    let file_create_attempted = Arc::new(Mutex::new(false));
    let continue_without = Arc::new(Mutex::new(false));
    let mut dialog_pushed = false;
    while !check_enable_file() && !*continue_without.lock().unwrap() {
        if !dialog_pushed {
            dialog_pushed = true;
            info!("CEF remote debugging not enabled in Steam, prompting user to enable...");
            let winit_waker = winit_waker.clone();
            let steam_path = Arc::new(steam_path.clone());
            let file_create_attempted = file_create_attempted.clone();
            let continue_without = continue_without.clone();
            let winit_waker2 = winit_waker.clone();

            #[cfg(target_os = "windows")]
            let extra_msg_string =
                String::from("This requires elevated permissions.\nA UAC prompt may be shown.\n\n");
            #[cfg(not(target_os = "windows"))]
            let extra_msg_string = String::from("");

            _ = push_dialog(Dialog::new_yes_no(
                "Enable Steam CEF debugging",
                (String::from(
                    "SISR requires advanced access to Steam for full functionality.
If you have launched via Steam, you can continue without,
but SISR may not work correctly and there is no support for this case.

Otherwise, choosing 'No' will cause SISR to quit.

Steam needs to be restarted.\n\n",
                ) + extra_msg_string.as_str())
                    + "Enable and restart Steam now?",
                move || {
                    let file_path = steam_path.join(".cef-enable-remote-debugging");
                    let exe = std::env::current_exe().unwrap();

                    debug!(
                        "Attempting to create CEF debug enable file at: {}",
                        file_path.display()
                    );
                    *file_create_attempted.lock().unwrap() = true;

                    #[cfg(target_os = "windows")]
                    let status = std::process::Command::new("powershell")
                        .arg("-ExecutionPolicy")
                        .arg("Bypass")
                        .arg("-Command")
                        .arg(format!(
                            "Start-Process '{}' -ArgumentList '--create-cef-file','\"{}\"' -Verb RunAs",
                            exe.display(),
                            file_path.display()
                        ))
                        .status();

                    #[cfg(not(target_os = "windows"))]
                    let status = std::fs::File::create(&file_path)
                        .map(|_| std::process::ExitStatus::default());

                    if status.map(|s| s.success()).unwrap_or(false) {
                        info!("CEF debug file created successfully");
                    } else {
                        warn!("Failed to create CEF debug file");
                        let winit_waker = winit_waker.clone();
                        std::thread::spawn(move || {
                            let winit_waker = winit_waker.clone();
                            std::thread::sleep(std::time::Duration::from_secs(1));
                            _ = push_dialog(Dialog::new_ok(
                                "Failed to enable CEF debugging",
                                "Failed to create CEF debug enable file in Steam directory.

Please create a file named '.cef-enable-remote-debugging' in your Steam installation directory manually.

SISR will close now.",
                                move || {
                                    let Ok(guard) = winit_waker.lock() else {
                                        panic!("Failed to acquire winit waker lock to quit app after missing steam")
                                    };
                                    let Some(proxy) = &*guard else {
                                        panic!("Winit waker not initialized to quit app after missing steam")
                                    };
                                    _ = proxy.send_event(RunnerEvent::Quit());
                                },
                            ));
                        });
                    }
                },
                move || {
                    if launched_via_steam() {
                        *continue_without.lock().unwrap() = true;
                        warn!(
                            "User chose not to enable CEF debugging, but SISR was launched via Steam. Continuing..."
                        );
                        return;
                    }
                    warn!("User chose not to enable CEF debugging");
                    let Ok(guard) = winit_waker2.lock() else {
                        panic!("Failed to acquire winit waker lock to quit app after missing steam")
                    };
                    let Some(proxy) = &*guard else {
                        panic!("Winit waker not initialized to quit app after missing steam")
                    };
                    _ = proxy.send_event(RunnerEvent::Quit());
                },
            ));
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
    if *file_create_attempted.lock().unwrap() && !*continue_without.lock().unwrap() {
        info!("CEF debugging enabled, restarting Steam...");
        _ = restart_steam(winit_waker.clone()).await;
    }

    if check_enable_file() && !*continue_without.lock().unwrap() {
        info!("checking if CEF debugging is reachable...");
        for _ in 0..10 {
            if check_enabled().await {
                info!("CEF debugging enabled and reachable");
                return (true, false);
            }
        }
        warn!("CEF debugging enable file present, but CEF debugging not reachable");
        let winit_waker = winit_waker.clone();
        std::thread::spawn(move || {
            let winit_waker = winit_waker.clone();
            std::thread::sleep(std::time::Duration::from_secs(1));
            _ = push_dialog(Dialog::new_ok(
                "CEF debugging not reachable",
                "CEF debugging is enabled, but SISR could not reach it.

Please make sure Steam is running and nothing is blocking or using connections to localhost:8080.
(Steam has hardcoded this port, cannot be changed (yet))

SISR will close now.",
                move || {
                    let Ok(guard) = winit_waker.lock() else {
                        panic!("Failed to acquire winit waker lock to quit app after missing steam")
                    };
                    let Some(proxy) = &*guard else {
                        panic!("Winit waker not initialized to quit app after missing steam")
                    };
                    _ = proxy.send_event(RunnerEvent::Quit());
                },
            ));
        });
        tokio::time::sleep(std::time::Duration::from_hours(9999)).await; // wait forever, we are done here
    }

    (check_enable_file(), *continue_without.lock().unwrap())
}

pub async fn restart_steam(winit_waker: Arc<Mutex<Option<EventLoopProxy<RunnerEvent>>>>) -> bool {
    if let Err(e) = open_steam_url("steam://exit") {
        let winit_waker = winit_waker.clone();
        error!("Failed to close Steam via URL scheme: {}", e);
        _ = push_dialog(Dialog::new_ok(
            "Failed to restart Steam",
            "Please restart Steam manually and restart SISR.",
            move || {
                let Ok(guard) = winit_waker.lock() else {
                    panic!("Failed to acquire winit waker lock to quit app after missing steam")
                };
                let Some(proxy) = &*guard else {
                    panic!("Winit waker not initialized to quit app after missing steam")
                };
                _ = proxy.send_event(RunnerEvent::Quit());
            },
        ));
        tokio::time::sleep(std::time::Duration::from_hours(99999)).await; // wait forever, basically, we are done here
    }
    info!("Waiting 5 seconds for Steam to close...");
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    info!("Restarting Steam...");
    if let Err(e) = open_steam_url("steam://open/main") {
        error!("Failed to start Steam via URL scheme: {}", e);
        let winit_waker = winit_waker.clone();
        _ = push_dialog(Dialog::new_ok(
            "Failed to restart Steam",
            "Please restart Steam manually and restart SISR.",
            move || {
                let Ok(guard) = winit_waker.lock() else {
                    panic!("Failed to acquire winit waker lock to quit app after missing steam")
                };
                let Some(proxy) = &*guard else {
                    panic!("Winit waker not initialized to quit app after missing steam")
                };
                _ = proxy.send_event(RunnerEvent::Quit());
            },
        ));
        tokio::time::sleep(std::time::Duration::from_hours(99999)).await; // wait forever, basically, we are done here
    }
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    ensure_steam_running(winit_waker.clone()).await
}

pub async fn ensure_steam_running(
    winit_waker: Arc<Mutex<Option<EventLoopProxy<RunnerEvent>>>>,
) -> bool {
    let Some(steam_path) = steam_path() else {
        error!("Steam installation path could not be found");
        let winit_waker = winit_waker.clone();
        _ = push_dialog(Dialog::new_ok(
            "Steam not found",
            "Steam installation could not be found.
SISR cannot function without Steam

SISR will now quit",
            move || {
                let Ok(guard) = winit_waker.lock() else {
                    panic!("Failed to acquire winit waker lock to quit app after missing steam")
                };
                let Some(proxy) = &*guard else {
                    panic!("Winit waker not initialized to quit app after missing steam")
                };
                _ = proxy.send_event(RunnerEvent::Quit());
            },
        ));
        return false;
    };
    debug!("Steam path: {}", steam_path.display());

    // I know... whatever

    let steam_running = crate::app::steam_utils::util::steam_running();
    let cancel = Arc::new(Mutex::new(false));
    let dialog_open = Arc::new(Mutex::new(false));
    while !steam_running && !*cancel.lock().unwrap() {
        if !*dialog_open.lock().unwrap() {
            debug!("Steam is not running, waiting for start...");
        }
        if !*dialog_open.lock().unwrap() {
            for _ in 0..CONFIG
                .get()
                .unwrap()
                .steam
                .steam_launch_timeout_secs
                .unwrap_or(1)
            {
                if crate::app::steam_utils::util::steam_running() {
                    info!("Steam is running");
                    return true;
                }
                trace!("Waiting for Steam to start...");
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        }
        if !steam_running {
            if !dialogs::REGISTRY
                .get()
                .unwrap()
                .snapshot_dialogs()
                .iter()
                .any(|d| d.title == "Steam not running")
            {
                *dialog_open.lock().unwrap() = true;
                let steam_path = Arc::new(steam_path.clone());
                let cancel = cancel.clone();
                let dialog_open = dialog_open.clone();
                _ = push_dialog(Dialog::new_yes_no(
                    "Steam not running",
                    "Start steam now?

Choosing 'No' will quit SISR",
                    move || {
                        info!("User chose to start steam, attempting to start...");
                        open_steam_url("steam://open/main").unwrap_or_else(|e| {
                            error!("Failed to start steam via URL scheme: {}", e);
                            if let Err(e) = Command::new(steam_path.join("steam.exe")).spawn() {
                                error!("Failed to start steam via executable: {}", e);
                            }
                        });
                        *dialog_open.lock().unwrap() = false; // whatever...
                    },
                    move || {
                        debug!("User chose not to start steam");
                        *cancel.lock().unwrap() = true;
                    },
                ));
            } else {
                // Dialog already open, just wait
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        }
    }
    info!("Steam is running: {}", steam_running);
    steam_running
}
