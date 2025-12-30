use std::net::ToSocketAddrs;
use std::process::Child;
use std::process::ExitCode;
use std::sync::{Arc, Mutex, OnceLock, RwLock};
use std::thread;
use tokio::sync::Notify;
use tracing::{debug, error, info, trace, warn};

use viiper_client::AsyncViiperClient;

use super::tray;
use super::window::WindowRunner;
use crate::app::gui::dialogs::{self, push_dialog};
use crate::app::gui::dispatcher::GuiDispatcher;
use crate::app::input::event::handler_events::HandlerEvent;
use crate::app::input::sdl_loop;
use crate::app::steam_utils::cef_debug;
use crate::app::steam_utils::cef_debug::ensure::{
    ensure_cef_enabled, ensure_steam_running,
};
use crate::app::steam_utils::cef_ws::WebSocketServer;
use crate::app::steam_utils::util::{
    launched_via_steam, load_steam_overlay, try_set_marker_steam_env,
};
use crate::app::window::{self, RunnerEvent};
use crate::app::{gui, signals, steam_utils};
use crate::config::{self, CONFIG};

static SPAWNED_VIIPER: RwLock<Option<Child>> = RwLock::new(None);
static TOKIO_HANDLE: OnceLock<tokio::runtime::Handle> = OnceLock::new();

pub fn get_tokio_handle() -> tokio::runtime::Handle {
    TOKIO_HANDLE
        .get()
        .cloned()
        .expect("TOKIO_HANDLE not initialized")
}

pub struct App {
    cfg: config::Config,
    gui_dispatcher: Option<Arc<Mutex<GuiDispatcher>>>,
}

impl App {


    pub fn new() -> Self {
        Self {
            cfg: CONFIG.read().expect("Failed to read CONFIG").as_ref().cloned().expect("Config not set"),
            gui_dispatcher: Some(Arc::new(Mutex::new(GuiDispatcher::new()))),
        }
    }

    pub fn run(&mut self) -> ExitCode {
        debug!("Running application...");
        debug!("Config: {:?}", self.cfg);


        let async_rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create async (tokio) runtime");
        TOKIO_HANDLE
            .set(async_rt.handle().clone())
            .expect("Failed to set TOKIO_HANDLE");

        gui::dialogs::REGISTRY
            .set(dialogs::Registry::new())
            .expect("Failed to init dialog registry");

        if !launched_via_steam() {
            match try_set_marker_steam_env() {
                Ok(_) => {
                    info!("Successfully set marker Steam environment variables");
                    load_steam_overlay();
                }
                Err(e) => {
                    error!("Failed to set marker Steam environment variables: {}", e);
                    // TODO: some error handling, whatever
                }
            }
        }

        let dispatcher = self.gui_dispatcher.clone();
        window::set_continuous_redraw(self.cfg.window.continous_draw.unwrap_or(false));
        window::set_kbm_emulation_enabled(self.cfg.kbm_emulation.unwrap_or(false));


        let viiper_address = self.cfg.viiper_address.as_ref().and_then(|addr_str| {
            addr_str
            .to_socket_addrs()
            .map_err(|e| error!("Invalid VIIPER address '{}': {}", addr_str, e))
            .ok()
            .and_then(|mut addrs| addrs.next())
        });
        let (sdl_ready_tx, sdl_ready_rx) = std::sync::mpsc::channel::<()>();

        let sdl_handle = thread::spawn(move || {   
            let mut input_loop = sdl_loop::InputLoop::new(viiper_address, dispatcher.expect("GUI dispatcher does not exist"));  
            let _ = sdl_ready_tx.send(());
            input_loop.run();
        });
        sdl_ready_rx.recv().expect("SDL thread died before signaling ready");

        let should_create_window = self.cfg.window.create.unwrap_or(true);
        let window_visible = Arc::new(Mutex::new(should_create_window));

        let fullscreen = self.cfg.window.fullscreen.unwrap_or(true);
        let initial_ui_visible = if fullscreen { false } else { !self.cfg.kbm_emulation.unwrap_or(false) };
        let ui_visible = Arc::new(Mutex::new(initial_ui_visible));

        let tray_handle = if self.cfg.tray.unwrap_or(true) {
            let window_visible_for_tray = window_visible.clone();
            let ui_visible_for_tray = ui_visible.clone();
            Some(thread::spawn(move || {
                tray::run(
                    window_visible_for_tray,
                    ui_visible_for_tray,
                );
            }))
        } else {
            None
        };

        if let Err(e) = signals::register_ctrlc_handler(move || {
            info!("Received Ctrl+C, shutting down...");
            Self::shutdown();
        }) {
            warn!("Failed to set Ctrl+C handler: {}", e);
        }

        let window_ready = Arc::new(Notify::new());
        self.ensure_viiper(
            window_ready.clone(),
        );
        self.steam_stuff(
            window_ready.clone(),
        );

        let mut window_runner = WindowRunner::new(
            self.gui_dispatcher.clone().expect("GUI dispatcher does not exist"),
            window_ready,
            window_visible.clone(),
            ui_visible.clone(),
        );

        let mut exit_code = window_runner.run();
        
        drop(window_runner);
        
        Self::shutdown();

        if let Err(e) = sdl_handle.join() {
            error!("SDL thread panicked: {:?}", e);
            exit_code = ExitCode::from(1);
        }

        if let Some(handle) = tray_handle
            && let Err(e) = handle.join()
        {
            error!("Tray thread panicked: {:?}", e);
            exit_code = ExitCode::from(1);
        }

        exit_code
    }

    pub fn shutdown(
    ) {
        use crate::app::steam_utils::util::open_steam_url;
        let _ = open_steam_url("steam://forceinputappid/0").inspect_err(|e| {
            error!("Failed to reset Steam input binding on shutdown: {}", e);
        });

        if let Some(mut child) = SPAWNED_VIIPER.write().expect("Failed to acquire spawned VIIPER lock").take() {
                trace!("Killing spawned VIIPER server");
                let _ = child.kill().inspect_err(|e|{
                    error!("Failed to kill spawned VIIPER process: {}", e);
                });
                let _ = child.wait().inspect_err(|e| {
                    error!("Failed to wait for spawned VIIPER process to exit: {}", e);
                });
        } else {
            debug!("No spawned VIIPER instance to kill");
        }


        debug!("Waking SDL event loop");
        if let Err(_e) = sdl_loop::get_event_sender().push_event(sdl3::event::Event::Quit { timestamp: 0 }) {
            // error!("Failed to push Quit event to SDL event loop: {}", e);
        }
    

        debug!("Waking winit event loop");
        if let Err(_e) = window::get_event_sender().send_event(RunnerEvent::Quit()) {
            // error!("Failed to push Quit event to winit event loop: {}", e);
        }
        
        tray::shutdown();
    }

    fn steam_stuff(
        &self,
        window_ready: Arc<Notify>,
    ) {
        get_tokio_handle().spawn(async move {
            window_ready.notified().await;
            let running = ensure_steam_running().await;
            if !running {
                error!("Steam ensure process failed, shutting down app");
                App::shutdown();
            }
            let (cef_enabled, continue_without) = ensure_cef_enabled().await;
            if !cef_enabled && !continue_without {
                error!("CEF enable process failed, shutting down app");
                App::shutdown();
            }
            if cef_enabled && !continue_without {
                info!("Starting WebSocket server...");
                let server = WebSocketServer::new().await;
                match server {
                    Ok((server, listener)) => {
                        let port = server.port();
                        info!("WebSocket server started on port {}", port);
                        server.run(
                            listener,
                        );
                        cef_debug::inject::set_ws_server_port(port);

                        trace!("Notifying SDL input handler of CEF debug readiness");

                        if let Err(e) = sdl_loop::get_event_sender().push_custom_event(HandlerEvent::CefDebugReady {
                            port
                        }) {
                            error!("Failed to notify SDL input handler of CEF debug readiness: {}", e);
                        }
                    }
                    Err(e) => {
                        error!("Failed to start WebSocket server: {}", e);
                    }
                }
            }

            let steam_path = steam_utils::util::steam_path();
            trace!("Steam path: {:?}", steam_path);
            let active_user_id = steam_utils::util::active_user_id();
            trace!("Active Steam user ID: {:?}", active_user_id);
            let mut marker_app_id: u32 = std::env::var("SISR_MARKER_ID").unwrap_or_default().parse().unwrap_or(0);
            if let Some(steam_path) = steam_path.clone()
                && let Some(user_id) = active_user_id
            {
                let Some(shortcuts_path) =
                    steam_utils::util::get_shortcuts_path(&steam_path, user_id)
                else {
                    warn!("Failed to determine Steam shortcuts.vdf path");
                    return;
                };
                trace!("Steam shortcuts.vdf path: {:?}", shortcuts_path);
                marker_app_id = steam_utils::util::shortcuts_has_sisr_marker(&shortcuts_path);
                info!(
                    "Steam shortcuts.vdf has SISR Marker shortcut with App ID: {}",
                    marker_app_id
                );
            } else {
                warn!(
                    "Steam path or active user ID not found; {:?}, {:?}",
                    steam_path, active_user_id
                );
            }
            if marker_app_id == 0 && !launched_via_steam() {
                _ = push_dialog(dialogs::Dialog::new_yes_no(
                    "SISR marker not found",
                    "SISR requires a marker in your Steam shortcuts
Would you like to create the marker shortcut now?
Steam MUST BE RUNNING and SISR will attempt to restart itself afterwards.

Selecting 'No' will exit SISR.",
                    move || {
                        get_tokio_handle().spawn(async move {
                            let marker_app_id = match steam_utils::util::create_sisr_marker_shortcut()
                                .await
                            {
                                Ok(app_id) => {
                                    info!(
                                        "Successfully created SISR marker shortcut with App ID: {}",
                                        app_id
                                    );
                                    app_id
                                }
                                Err(e) => {
                                    error!("Failed to create SISR marker shortcut: {}", e);
                                    0
                                }
                            };
                            if marker_app_id != 0 {
                                let executable_path = std::env::current_exe().expect("Failed to get current exe path");
                                let args: Vec<String> = std::env::args().skip(1).collect();
                                
                                #[cfg(target_os = "windows")]
                                {
                                    use std::os::windows::process::CommandExt;
                                    let _ = std::process::Command::new(&executable_path)
                                        .args(&args)
                                        // .creation_flags(0x00000008) // CREATE_NO_WINDOW
                                        .creation_flags(0x00000200) // CREATE_NEW_PROCESS_GROUP
                                        .spawn();
                                }
                                
                                #[cfg(target_os = "linux")]
                                {
                                    use std::os::unix::process::CommandExt;
                                    let _ = std::process::Command::new(&executable_path)
                                        .args(&args)
                                        .exec();
                                }
                                std::process::exit(0);
                            } 
                            _ = push_dialog(dialogs::Dialog::new_ok(
                                "Create Marker Shortcut", 
                                "Failed to create SISR marker shortcut.
Please create a shortcut to SISR with the launch argument '--marker' in your Steam shortcuts manually.

The application will now exit.", ||{
                                std::process::exit(1);
                            }))
                        });
                    },
                    || {
                        std::process::exit(1);
                    },
                ))
            }        });
    }

    fn ensure_viiper(&self,
        window_ready: Arc<Notify>,
    ) {
        get_tokio_handle().spawn(async move {
            async fn show_dialog_and_quit(
                ui_ready: Arc<Notify>,
                title: &'static str,
                message: String,
            ) {
                ui_ready.notified().await;

                let _ = push_dialog(dialogs::Dialog::new_ok(title, message, move || {
                    App::shutdown();
                }))
                .inspect_err(|e| error!("Failed to push dialog: {}", e));
            }

            let addr = CONFIG
                .read()
                .ok()
                .and_then(|g| g.as_ref().and_then(|cfg| cfg.viiper_address.clone()))
                .and_then(|s| s.to_socket_addrs().ok().and_then(|mut a| a.next()))
                .unwrap_or_else(|| "localhost:3242".to_socket_addrs().unwrap().next().unwrap());

            let retry_schedule = [
                std::time::Duration::from_secs(1),
                std::time::Duration::from_secs(1),
                std::time::Duration::from_secs(2),
                std::time::Duration::from_secs(4),
                std::time::Duration::from_secs(6),
            ];

            let client = AsyncViiperClient::new(addr);
            
            #[cfg(not(target_os = "linux"))]
            let mut spawn_attempted = false;

            for (attempt, delay) in retry_schedule.into_iter().enumerate() {
                match client.ping().await {
                    Ok(resp) => {
                        let is_viiper = resp.server == "VIIPER";
                        if !is_viiper {
                            let msg = format!(
                                "A non-VIIPER server is running at {addr} (server={}).\n\nSISR requires VIIPER to function and will now exit.",
                                resp.server
                            );
                            error!("{}", msg.replace('\n', " | "));
                            show_dialog_and_quit(
                                window_ready.clone(),
                                "Invalid VIIPER server",
                                msg,
                            )
                            .await;
                            return;
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
                            error!("{}", msg.replace('\n', " | "));
                            show_dialog_and_quit(
                                window_ready.clone(),
                                "VIIPER too old",
                                msg,
                            )
                            .await;
                            return;
                        }

                        info!("VIIPER is ready (version={})", version);

                        window_ready.notified().await;
                        trace!("Notifying SDL input handler of VIIPER readiness");

                        if let Err(e) = sdl_loop::get_event_sender().push_custom_event(HandlerEvent::ViiperReady {
                            version: version.clone(),
                        }) {
                            error!("Failed to notify SDL input handler of VIIPER readiness: {}", e);
                        }
                        return;
                    }
                    Err(e) => {
                        warn!("VIIPER ping failed (attempt {}): {}", attempt + 1, e);

                        #[cfg(not(target_os = "linux"))]
                        if addr.ip().is_loopback() && !spawn_attempted {
                            spawn_attempted = true;

                            let spawn_res: anyhow::Result<()> = (|| {
                                let mut child_opt = SPAWNED_VIIPER.write().expect("Failed to acquire spawned VIIPER lock");
                                if child_opt.is_some()
                                {
                                    return Ok(());
                                }

                                let exe_dir = std::env::current_exe()
                                    .ok()
                                    .and_then(|p| p.parent().map(|p| p.to_path_buf()))
                                    .unwrap_or_else(|| PathBuf::from("."));
                                let viiper_path = exe_dir.join(if cfg!(windows) { "viiper.exe" } else { "viiper" });
                                if !viiper_path.exists() {
                                    anyhow::bail!(
                                        "VIIPER executable not found at {}\nExpected it next to SISR.",
                                        viiper_path.display()
                                    );
                                }

                                let log_path =  directories::ProjectDirs::from("", "", "SISR")
                                    .map(|proj_dirs| proj_dirs.data_dir().join("VIIPER.log"));
                                info!("Starting local VIIPER: {}", viiper_path.display());

                                let mut cmd = Command::new(&viiper_path);
                                cmd.arg("server");
                                if let Some(log_path) = &log_path {
                                    cmd.arg("--log.file")
                                    .arg(log_path);
                                }
                                cmd.stdin(std::process::Stdio::null())
                                    .stdout(std::process::Stdio::null())
                                    .stderr(std::process::Stdio::null());

                                let child = cmd.spawn().inspect_err(|e| {
                                    error!(
                                        "VIIPER spawn failed: {}",
                                        e
                                    );
                                })?;
                                info!("Spawned VIIPER pid={}", child.id());
                                *child_opt = Some(child);

                                Ok(())
                            })();

                            if let Err(spawn_err) = spawn_res {
                                let msg = format!(
                                    "Failed to start VIIPER locally.\n\n{spawn_err}\n\nSISR will now exit."
                                );
                                error!("{}", msg.replace('\n', " | "));
                                show_dialog_and_quit(
                                    window_ready.clone(),
                                    "Failed to start VIIPER",
                                    msg,
                                )
                                .await;
                                return;
                            }
                        }

                        tokio::time::sleep(delay).await;
                    }
                }
            }

            #[cfg(target_os = "linux")]
            let msg = format!(
                "Unable to connect to VIIPER at {addr} after multiple attempts.\n\n\
                On Linux, VIIPER must be installed as a system service.\n\
                See installation instructions at:\n\
                https://alia5.github.io/SISR/stable/getting-started/installation/\n\n\
                SISR will now exit."
            );
            
            #[cfg(not(target_os = "linux"))]
            let msg = format!(
                "Unable to connect to VIIPER at {addr} after multiple attempts.\n\nSISR will now exit."
            );
            
            error!("{}", msg.replace('\n', " | "));
            show_dialog_and_quit(
                window_ready,
                "VIIPER unavailable",
                msg,
            )
            .await;
        });
    }

}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
