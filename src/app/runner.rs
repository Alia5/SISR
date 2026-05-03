use std::{
    net::ToSocketAddrs,
    process::{Child, ExitCode},
    sync::{Arc, Mutex, OnceLock, RwLock, mpsc},
    thread,
    time::Duration,
};

use crate::{
    app::{
        api::{self, handler::connect_viiper::SPAWNED_VIIPER}, hid_hooks,
        input::{context::Context, sdl_loop},
        signals, steam, tray,
        window::{self, event::WindowRunnerEvent, runner::WindowRunner},
    },
    config::get_config,
};
static TOKIO_HANDLE: OnceLock<tokio::runtime::Handle> = OnceLock::new();

pub fn get_tokio_handle() -> tokio::runtime::Handle {
    TOKIO_HANDLE
        .get()
        .cloned()
        .expect("TOKIO_HANDLE not initialized")
}

const CLEANUP_SCRIPT: &str = include_str!("../../CEF_Payloads/dist/cleanup.js");

#[derive(Default)]
pub struct AppRunner {}

impl AppRunner {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(&mut self) -> ExitCode {
        tracing::debug!("Running application...");
        let cfg = get_config();
        tracing::debug!("Config: {:?}", cfg);

        let async_rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create async (tokio) runtime");
        TOKIO_HANDLE
            .set(async_rt.handle().clone())
            .expect("Failed to set TOKIO_HANDLE");

        hid_hooks::hid_check::enumerate_hid_exports();

        if !steam::util::launched_via_steam() && !cfg.steam.no_steam.unwrap_or(false) {
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

        let viiper_address = cfg.viiper_address.as_ref().and_then(|addr_str| {
            addr_str
                .to_socket_addrs()
                .map_err(|e| tracing::error!("Invalid VIIPER address '{}': {}", addr_str, e))
                .ok()
                .and_then(|mut addrs| addrs.next())
        });
        let (sdl_ready_tx, sdl_ready_rx) = std::sync::mpsc::channel::<()>();

        tracing::debug!("Spawning SDL thread...");

        let (ctx_tx, ctx_rx) = mpsc::channel::<Arc<Mutex<Context>>>();
        let sdl_handle = thread::spawn(move || {
            let mut input_loop = sdl_loop::InputLoop::new(viiper_address);

            let ctx = input_loop.get_ctx();
            let _ = ctx_tx.send(ctx);

            let _ = sdl_ready_tx.send(());
            input_loop.run();
        });
        sdl_ready_rx
            .recv()
            .expect("SDL thread died before signaling ready");

        tracing::debug!("Spawning tray thread...");
        let tray_handle = if cfg.tray.unwrap_or(true) {
            Some(thread::spawn(move || {
                tray::run();
            }))
        } else {
            None
        };

        let sdl_loop_ctx = ctx_rx
            .recv()
            .expect("SDL thread died before sending Context");

        tracing::debug!("Spawning API server...");
        let ctx_for_api = sdl_loop_ctx.clone();
        async_rt.spawn(async {
            if let Err(e) = api::listen_and_serve(ctx_for_api).await {
                tracing::error!("API server error: {}", e);
                Self::shutdown();
            }
        });

        tracing::debug!("Registering Ctrl+C handler...");
        if let Err(e) = signals::register_ctrlc_handler(move || {
            tracing::info!("Received Ctrl+C, shutting down...");
            Self::shutdown();
        }) {
            tracing::warn!("Failed to set Ctrl+C handler: {}", e);
        }

        tracing::debug!("Spawning window runner...");
        let mut window_runner = WindowRunner::new(sdl_loop_ctx);
        let mut exit_code = window_runner.run();

        drop(window_runner);

        Self::shutdown();

        if let Err(e) = sdl_handle.join() {
            tracing::error!("SDL thread panicked: {:?}", e);
            exit_code = ExitCode::from(1);
        }

        if let Some(handle) = tray_handle
            && let Err(e) = handle.join()
        {
            tracing::error!("Tray thread panicked: {:?}", e);
            exit_code = ExitCode::from(1);
        }

        exit_code
    }

    pub fn shutdown() {
        tracing::info!("Shutting down application...");

        let (cleanup_done_tx, cleanup_done_rx) = mpsc::channel::<()>();
        get_tokio_handle().spawn(async move {
            for tab in steam::cef_inject::injector::CLEANUP_TABS {
                if let Err(e) =
                    steam::cef_inject::injector::inject_into_tab::<()>(tab, CLEANUP_SCRIPT).await
                {
                    tracing::error!(
                        "Failed to inject cleanup script into Steam CEF tab '{}': {}",
                        tab,
                        e
                    );
                } else {
                    tracing::trace!(
                        "Successfully injected cleanup script into Steam CEF tab '{}'",
                        tab
                    );
                }
            }

            let _ = cleanup_done_tx.send(());
        });

        match cleanup_done_rx.recv_timeout(Duration::from_secs(2)) {
            Ok(()) => {
                tracing::trace!("Cleanup injection finished before shutdown");
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                tracing::warn!(
                    "Timed out waiting for cleanup injection; continuing shutdown"
                );
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                tracing::warn!("Cleanup task ended unexpectedly before signaling completion");
            }
        }

        if steam::util::steam_running() {
            let _ = steam::util::open_url("steam://forceinputappid/0").inspect_err(|e| {
                tracing::error!("Failed to reset Steam input binding on shutdown: {}", e);
            });
        }

        if let Some(mut child) = SPAWNED_VIIPER
            .write()
            .expect("Failed to acquire spawned VIIPER lock")
            .take()
        {
            tracing::trace!("Killing spawned VIIPER server");
            let _ = child.kill().inspect_err(|e| {
                tracing::error!("Failed to kill spawned VIIPER process: {}", e);
            });
            let _ = child.wait().inspect_err(|e| {
                tracing::error!("Failed to wait for spawned VIIPER process to exit: {}", e);
            });
        } else {
            tracing::debug!("No spawned VIIPER instance to kill");
        }

        tracing::debug!("Waking SDL event loop for shutdown");
        if let Err(_e) =
            sdl_loop::get_event_sender().push_event(sdl3::event::Event::Quit { timestamp: 0 })
        {
            // error!("Failed to push Quit event to SDL event loop: {}", e);
        }

        tracing::debug!("Waking winit event loop for shutdown");
        if let Err(_e) = window::event::get_event_sender().send_event(WindowRunnerEvent::Quit()) {
            // error!("Failed to push Quit event to winit event loop: {}", e);
        }
        tray::shutdown();
    }
}
