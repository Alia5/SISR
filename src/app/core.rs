use sdl3::event::EventSender;
use std::net::ToSocketAddrs;
use std::process::ExitCode;
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::sync::Notify;
use tracing::{debug, error, info, warn};
use winit::event_loop::EventLoopProxy;

use super::tray;
use super::window::WindowRunner;
use crate::app::gui::dispatcher::GuiDispatcher;
use crate::app::input::{self, sdl};
use crate::app::signals;
use crate::app::steam_utils::cef_debug;
use crate::app::steam_utils::cef_debug::ensure::{ensure_cef_enabled, ensure_steam_running};
use crate::app::steam_utils::cef_ws::WebSocketServer;
use crate::app::window::RunnerEvent;
use crate::config;

pub struct App {
    cfg: config::Config,
    sdl_waker: Arc<Mutex<Option<EventSender>>>,
    winit_waker: Arc<Mutex<Option<EventLoopProxy<RunnerEvent>>>>,
    gui_dispatcher: Arc<Mutex<Option<GuiDispatcher>>>,
}

impl App {
    pub fn new() -> Self {
        Self {
            cfg: config::CONFIG.get().cloned().expect("Config not set"),
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

        let sdl_waker = self.sdl_waker.clone();
        let winit_waker_for_sdl = self.winit_waker.clone();
        let dispatcher = self.gui_dispatcher.clone();

        let input_loop = Arc::new(Mutex::new(Some(input::sdl::InputLoop::new(
            sdl_waker,
            winit_waker_for_sdl,
            dispatcher,
            async_rt.handle().clone(),
        ))));

        let should_create_window = self.cfg.window.create.unwrap_or(true);
        let window_visible = Arc::new(Mutex::new(should_create_window));

        let tray_handle = if self.cfg.tray.unwrap_or(true) {
            let sdl_waker_for_tray = self.sdl_waker.clone();
            let winit_waker_for_tray = self.winit_waker.clone();
            let window_visible_for_tray = window_visible.clone();
            Some(thread::spawn(move || {
                tray::run(
                    sdl_waker_for_tray,
                    winit_waker_for_tray,
                    window_visible_for_tray,
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
            self.gui_dispatcher.clone(),
            window_ready,
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
                    Ok(server) => {
                        let port = server.port();
                        info!("WebSocket server started on port {}", port);
                        server.run(async_handle, winit_waker, sdl_waker);
                        cef_debug::inject::set_ws_server_port(port);


                    }
                    Err(e) => {
                        error!("Failed to start WebSocket server: {}", e);
                    }
                }
            }
        });
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
