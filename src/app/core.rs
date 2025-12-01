use sdl3::event::EventSender;
use std::net::ToSocketAddrs;
use std::process::ExitCode;
use std::sync::{Arc, Mutex};
use std::thread;
use tracing::{debug, error, info, warn};
use winit::event_loop::EventLoopProxy;

use super::tray;
use super::window::WindowRunner;
use crate::app::gui::dispatcher::GuiDispatcher;
use crate::app::input::{self};
use crate::app::window::RunnerEvent;
use crate::config;

pub struct App {
    cfg: config::Config,
    sdl_waker: Arc<Mutex<Option<EventSender>>>,
    winit_waker: Arc<Mutex<Option<EventLoopProxy<RunnerEvent>>>>,
    gui_dispatcher: Arc<Mutex<Option<GuiDispatcher>>>,
}

impl App {
    pub fn new(cfg: config::Config) -> Self {
        Self {
            cfg,
            sdl_waker: Arc::new(Mutex::new(None)),
            winit_waker: Arc::new(Mutex::new(None)),
            gui_dispatcher: Arc::new(Mutex::new(None)),
        }
    }

    pub fn run(&mut self) -> ExitCode {
        debug!("Running application...");
        debug!("Config: {:?}", self.cfg);

        let sdl_waker = self.sdl_waker.clone();
        let winit_waker_for_sdl = self.winit_waker.clone();
        let dispatcher = self.gui_dispatcher.clone();

        let input_loop = Arc::new(Mutex::new(Some(input::sdl::InputLoop::new(
            sdl_waker,
            winit_waker_for_sdl,
            dispatcher,
        ))));

        let tray_handle = if self.cfg.tray.unwrap_or(true) {
            let sdl_waker_for_tray = self.sdl_waker.clone();
            let winit_waker_for_tray = self.winit_waker.clone();
            Some(thread::spawn(move || {
                tray::run(sdl_waker_for_tray, winit_waker_for_tray);
            }))
        } else {
            None
        };

        let sdl_waker_for_ctrlc = self.sdl_waker.clone();
        let winit_waker_for_ctrlc = self.winit_waker.clone();
        if let Err(e) = ctrlc::set_handler(move || {
            info!("Received Ctrl+C, shutting down...");
            Self::shutdown(Some(&sdl_waker_for_ctrlc), Some(&winit_waker_for_ctrlc));
        }) {
            warn!("Failed to set Ctrl+C handler: {}", e);
        }

        let mut exit_code = ExitCode::SUCCESS;

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
        let sdl_handle: thread::JoinHandle<()>;
        let should_create_window = self.cfg.window.create.unwrap_or(true);
        if should_create_window {
            match self.gui_dispatcher.lock() {
                Ok(mut guard) => {
                    *guard = Some(GuiDispatcher::new(self.winit_waker.clone()));
                }
                Err(e) => {
                    error!("Failed to initialize GUI dispatcher: {}", e);
                }
            }

            sdl_handle = thread::spawn(create_sdl_handle);

            let mut window_runner =
                WindowRunner::new(self.winit_waker.clone(), self.gui_dispatcher.clone());
            exit_code = window_runner.run();
            Self::shutdown(Some(&self.sdl_waker), Some(&self.winit_waker));
        } else {
            info!("Running without window. Press Ctrl+C to exit.");
            sdl_handle = thread::spawn(create_sdl_handle);
        }

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
}
