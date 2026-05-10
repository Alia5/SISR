use std::mem::{Discriminant, discriminant};

use winit::event_loop::ActiveEventLoop;

use crate::app::window::{
    event::WindowRunnerEvent, handler::router::EventHandler, runner::WindowRunner
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
        tracing::trace!("SetKbmCursorGrabHandler received event: {:?}", event);

        let enabled = match event {
            WindowRunnerEvent::SetKbmCursorGrab(enabled) => *enabled,
            _ => return,
        };
        runner.set_kbm_emulation_enabled(enabled);
        let Some(window) = runner.get_window() else {
            tracing::warn!("wtf? no window.");
            return;
        };
        if window.is_visible() == Some(false) {
            return;
        }
        runner.recalculate_passthrough();
        if !runner.get_webview().is_some_and(|wv| wv.is_visible()) {
            if enabled {
                if let Err(e) = window.set_cursor_grab(winit::window::CursorGrabMode::Confined) {
                    tracing::warn!("Failed to confine cursor to window: {e}");
                }
            } else {
                _ = window.set_cursor_grab(winit::window::CursorGrabMode::None);
            }
            runner.update_cursor_visibility();
        } else if let Err(e) = window.set_cursor_grab(winit::window::CursorGrabMode::Confined) {
            tracing::warn!("Failed to confine cursor to window: {e}");
        }
    }

    fn listen_events(&self) -> Vec<Discriminant<WindowRunnerEvent>> {
        vec![discriminant(&WindowRunnerEvent::SetKbmCursorGrab(true))]
    }
}
