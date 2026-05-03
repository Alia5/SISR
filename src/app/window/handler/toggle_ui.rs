use std::mem::{Discriminant, discriminant};

use winit::event_loop::ActiveEventLoop;
use winit::window::{Fullscreen, WindowLevel};

use crate::{
    app::{
        tray::{self, event::TrayEvent},
        window::{
            event::{WindowRunnerEvent, get_event_sender},
            handler::router::EventHandler,
            runner::WindowRunner,
        },
    },
    config::get_config,
};

#[derive(Default)]
pub struct Handler {}

impl EventHandler for Handler {
    fn handle_event(
        &self,
        runner: &mut WindowRunner,
        _event_loop: &ActiveEventLoop,
        event: &WindowRunnerEvent,
    ) {
        tracing::trace!("ToggleUiHandler received event: {:?}", event);

        let show = match event {
            WindowRunnerEvent::ToggleUi(show) => *show,
            _ => return,
        };

        let Some(window) = runner.get_window() else {
            tracing::warn!("wtf? no window.");
            return;
        };
        let Some(wv) = runner.get_webview_mut() else {
            tracing::warn!("Webview not initialized");
            return;
        };

        let win_create = get_config().window.create.unwrap_or(false);
        let fullscreen = get_config().window.fullscreen.unwrap_or(true);

        let should_hide_window = !win_create || !fullscreen;

        if !show {
            tracing::info!("Hiding UI");
            wv.hide();
            if fullscreen {
                window.set_fullscreen(Some(Fullscreen::Borderless(None)));
                window.set_decorations(false);
                window.set_window_level(WindowLevel::AlwaysOnTop);
            }
            if should_hide_window {
                // fuck clippy
                if let Err(e) = get_event_sender().send_event(WindowRunnerEvent::HideWindow()) {
                    tracing::error!("Failed to send HideWindow event: {:?}", e);
                }
            }
            runner.set_passthrough(true);
        } else {
            tracing::info!("Showing UI");
            if fullscreen {
                window.set_fullscreen(None);
                window.set_decorations(true);
                window.set_window_level(WindowLevel::Normal);
                let _ = window.request_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0));
            }
            wv.show();
            if !window.is_visible().unwrap_or(false) {
                // fuck clippy
                if let Err(e) = get_event_sender().send_event(WindowRunnerEvent::ShowWindow()) {
                    tracing::error!("Failed to send ShowWindow event: {:?}", e);
                }
            }
            runner.set_passthrough(false);
        }
        if let Err(e) =
            tray::event::get_event_sender().send(TrayEvent::SetWindowState(show))
        {
            tracing::error!("Failed to send SetWindowState event: {:?}", e);
        }
    }

    fn listen_events(&self) -> Vec<Discriminant<WindowRunnerEvent>> {
        vec![discriminant(&WindowRunnerEvent::ToggleUi(false))]
    }
}
