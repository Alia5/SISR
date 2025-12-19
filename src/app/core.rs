use sdl3::event::EventSender;
use std::net::ToSocketAddrs;
use std::process::ExitCode;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::sync::Notify;
use tracing::{debug, error, info, trace, warn};
use winit::event_loop::EventLoopProxy;

use super::tray;
use super::window::WindowRunner;
use crate::app::gui::dialogs::{self, push_dialog};
use crate::app::gui::dispatcher::GuiDispatcher;
use crate::app::input::handler::HandlerEvent;
use crate::app::input::{self};
use crate::app::steam_utils::cef_debug;
use crate::app::steam_utils::cef_debug::ensure::{
    ensure_cef_enabled, ensure_steam_running,
};
use crate::app::steam_utils::cef_ws::WebSocketServer;
use crate::app::steam_utils::util::{
    launched_via_steam, load_steam_overlay, try_set_marker_steam_env,
};
use crate::app::window::RunnerEvent;
use crate::app::{gui, signals, steam_utils};
use crate::config::{self, CONFIG};

pub struct App {
    cfg: config::Config,
    sdl_waker: Arc<Mutex<Option<EventSender>>>,
    winit_waker: Arc<Mutex<Option<EventLoopProxy<RunnerEvent>>>>,
    gui_dispatcher: Arc<Mutex<Option<GuiDispatcher>>>,
}

impl App {
    pub fn new() -> Self {
        Self {
            cfg: CONFIG.read().expect("Failed to read CONFIG").as_ref().cloned().expect("Config not set"),
            sdl_waker: Arc::new(Mutex::new(None)),
            winit_waker: Arc::new(Mutex::new(None)),
            gui_dispatcher: Arc::new(Mutex::new(None)),
        }
    }

    pub fn run(&mut self) -> ExitCode {
        debug!("Running application...");
        debug!("Config: {:?}", self.cfg);

        let async_rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create async (tokio) runtime");

        gui::dialogs::REGISTRY
            .set(dialogs::Registry::new(self.winit_waker.clone()))
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

        let sdl_waker = self.sdl_waker.clone();
        let winit_waker_for_sdl = self.winit_waker.clone();
        let dispatcher = self.gui_dispatcher.clone();
        let continuous_redraw = Arc::new(AtomicBool::new(
            self.cfg.window.continous_draw.unwrap_or(false),
        ));

        let kbm_emulation_enabled = Arc::new(AtomicBool::new(
            self.cfg.kbm_emulation.unwrap_or(false),
        ));

        let input_loop = Arc::new(Mutex::new(Some(input::sdl::InputLoop::new(
            sdl_waker,
            winit_waker_for_sdl,
            dispatcher,
            async_rt.handle().clone(),
            continuous_redraw.clone(),
            kbm_emulation_enabled.clone(),
        ))));

        let should_create_window = self.cfg.window.create.unwrap_or(true);
        let window_visible = Arc::new(Mutex::new(should_create_window));

        let fullscreen = self.cfg.window.fullscreen.unwrap_or(true);
        let initial_ui_visible = if fullscreen { false } else { !self.cfg.kbm_emulation.unwrap_or(false) };
        let ui_visible = Arc::new(Mutex::new(initial_ui_visible));

        let tray_handle = if self.cfg.tray.unwrap_or(true) {
            let sdl_waker_for_tray = self.sdl_waker.clone();
            let winit_waker_for_tray = self.winit_waker.clone();
            let window_visible_for_tray = window_visible.clone();
            let ui_visible_for_tray = ui_visible.clone();
            let handle_for_tray = async_rt.handle().clone();
            let kbm_emulation_enabled_for_tray = kbm_emulation_enabled.clone();
            Some(thread::spawn(move || {
                tray::run(
                    sdl_waker_for_tray,
                    winit_waker_for_tray,
                    window_visible_for_tray,
                    ui_visible_for_tray,
                    handle_for_tray,
                    kbm_emulation_enabled_for_tray,
                );
            }))
        } else {
            None
        };

        let sdl_waker_for_ctrlc = self.sdl_waker.clone();
        let winit_waker_for_ctrlc = self.winit_waker.clone();
        if let Err(e) = signals::register_ctrlc_handler(move || {
            info!("Received Ctrl+C, shutting down...");
            Self::shutdown(Some(&sdl_waker_for_ctrlc), Some(&winit_waker_for_ctrlc));
        }) {
            warn!("Failed to set Ctrl+C handler: {}", e);
        }

        let viiper_address = self.cfg.viiper_address.as_ref().and_then(|addr_str| {
            addr_str
                .to_socket_addrs()
                .map_err(|e| error!("Invalid VIIPER address '{}': {}", addr_str, e))
                .ok()
                .and_then(|mut addrs| addrs.next())
        });

        let create_sdl_handle = move || {
            if let Ok(mut guard) = input_loop.lock()
                && let Some(mut input_loop) = guard.take()
            {
                input_loop.run(viiper_address);
            }
        };
        let sdl_handle = thread::spawn(create_sdl_handle);
        match self.gui_dispatcher.lock() {
            Ok(mut guard) => {
                *guard = Some(GuiDispatcher::new(self.winit_waker.clone()));
            }
            Err(e) => {
                error!("Failed to initialize GUI dispatcher: {}", e);
            }
        }

        let window_ready = Arc::new(Notify::new());
        self.steam_stuff(
            async_rt.handle().clone(),
            self.winit_waker.clone(),
            self.sdl_waker.clone(),
            window_ready.clone(),
        );

        let mut window_runner = WindowRunner::new(
            self.winit_waker.clone(),
            self.sdl_waker.clone(),
            self.gui_dispatcher.clone(),
            window_ready,
            continuous_redraw,
            kbm_emulation_enabled.clone(),
            window_visible.clone(),
            ui_visible.clone(),
        );

        let mut exit_code = window_runner.run();
        Self::shutdown(Some(&self.sdl_waker), Some(&self.winit_waker));

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
        sdl_waker: Option<&Arc<Mutex<Option<EventSender>>>>,
        winit_waker: Option<&Arc<Mutex<Option<EventLoopProxy<RunnerEvent>>>>>,
    ) {
        if let Some(sdl_waker) = sdl_waker
            && let Ok(guard) = sdl_waker.lock()
            && let Some(sender) = &*guard
        {
            debug!("Waking SDL event loop");
            _ = sender.push_event(sdl3::event::Event::Quit { timestamp: 0 })
        }
        if let Some(winit_waker) = winit_waker
            && let Ok(guard) = winit_waker.lock()
            && let Some(proxy) = &*guard
        {
            debug!("Waking winit event loop");
            _ = proxy.send_event(RunnerEvent::Quit());
        }
        tray::shutdown();
    }

    fn steam_stuff(
        &self,
        async_handle: tokio::runtime::Handle,
        winit_waker: Arc<Mutex<Option<EventLoopProxy<RunnerEvent>>>>,
        sdl_waker: Arc<Mutex<Option<EventSender>>>,
        window_ready: Arc<Notify>,
    ) {
        async_handle.clone().spawn(async move {
            window_ready.notified().await;
            let running = ensure_steam_running(winit_waker.clone()).await;
            if !running {
                error!("Steam ensure process failed, shutting down app");
                App::shutdown(None, Some(&winit_waker));
            }
            let (cef_enabled, continue_without) = ensure_cef_enabled(winit_waker.clone()).await;
            if !cef_enabled && !continue_without {
                error!("CEF enable process failed, shutting down app");
                App::shutdown(None, Some(&winit_waker));
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
                            async_handle.clone(),
                            winit_waker.clone(),
                            sdl_waker.clone(),
                        );
                        cef_debug::inject::set_ws_server_port(port);

                        let Ok(sdl_waker) = sdl_waker.lock() else {
                            error!("Failed to lock SDL waker to notify CEF debug readiness");
                            return;
                        };
                        sdl_waker.as_ref().and_then(|sender| {
                            trace!("Notifying SDL input handler of CEF debug readiness");
                            sender
                                .push_custom_event(HandlerEvent::CefDebugReady { port })
                                .ok()
                        });
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
                let handle_clone = async_handle.clone();
                _ = push_dialog(dialogs::Dialog::new_yes_no(
                    "SISR marker not found",
                    "SISR requires a marker in your Steam shortcuts
Would you like to create the marker shortcut now?
Steam MUST BE RUNNING and SISR will attempt to restart itself afterwards.

Selecting 'No' will exit SISR.",
                    move || {
                        handle_clone.clone().spawn(async move {
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
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
