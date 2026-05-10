use std::mem::{Discriminant, discriminant};

use winit::event_loop::ActiveEventLoop;
use winit::window::CursorGrabMode;

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

        let Some(window) = runner.get_window() else {
            tracing::warn!("wtf? no window.");
            return;
        };
        let win_create = get_config().window.create.unwrap_or(false);
        let fullscreen = get_config().window.fullscreen.unwrap_or(true);
        let kbm_enabled = runner.is_kbm_enabled();

        let show = match event {
            WindowRunnerEvent::ToggleUi(Some(show)) => *show,
            WindowRunnerEvent::ToggleUi(None) => !runner.get_webview().map(|wv| wv.is_visible()).unwrap_or(false),
            _ => return,
        };

        // When KBM is active the window must stay visible for cursor-grab to work;
        // only the webview should be toggled.
        let should_hide_window = (!win_create || !fullscreen) && !kbm_enabled;

        if !show {
            tracing::info!("Hiding UI");
            if let Some(wv) = runner.get_webview_mut() {
                wv.hide();
            }
            if fullscreen {
                runner.set_fullscreen(true);
            }
            if should_hide_window {
                // fuck clippy
                if let Err(e) = get_event_sender().send_event(WindowRunnerEvent::HideWindow()) {
                    tracing::error!("Failed to send HideWindow event: {:?}", e);
                }
            }

            runner.recalculate_passthrough();
            if kbm_enabled {
                if let Err(e) = window.set_cursor_grab(CursorGrabMode::Confined) {
                    tracing::warn!("Failed to confine cursor to window: {e}");
                }
                runner.update_cursor_visibility();
            }
        } else {
            tracing::info!("Showing UI");
            if fullscreen {
                runner.set_fullscreen(false);
                // let _ = window.request_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0));
            }
            if let Some(wv) = runner.get_webview_mut() {
                wv.show();
            }
            if !window.is_visible().unwrap_or(false) {
                // fuck clippy
                if let Err(e) = get_event_sender().send_event(WindowRunnerEvent::ShowWindow()) {
                    tracing::error!("Failed to send ShowWindow event: {:?}", e);
                }
            }
            runner.set_passthrough(false);
            _ = window.set_cursor_grab(winit::window::CursorGrabMode::None);
            runner.update_cursor_visibility();
        }
        if let Err(e) =
            tray::event::get_event_sender().send(TrayEvent::SetWindowState(show))
        {
            tracing::error!("Failed to send SetWindowState event: {:?}", e);
        }
    }

    fn listen_events(&self) -> Vec<Discriminant<WindowRunnerEvent>> {
        vec![discriminant(&WindowRunnerEvent::ToggleUi(None))]
    }
}
