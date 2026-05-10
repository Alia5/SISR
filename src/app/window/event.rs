use std::sync::{Arc, OnceLock};

use winit::event_loop::EventLoopProxy;

#[derive(Debug)]
pub enum WindowRunnerEvent {
    Quit(),
    Redraw(),
    ShowWindow(),
    HideWindow(),
    ToggleUi(Option<bool>),
    SetFullscreen(bool),
    EnterCaptureMode(),
    SetKbmCursorGrab(bool),
    OverlayStateChanged(bool),
    InvalidateSvelteState(),
}

pub static EVENT_SENDER: OnceLock<Arc<EventLoopProxy<WindowRunnerEvent>>> = OnceLock::new();

pub fn get_event_sender() -> Arc<EventLoopProxy<WindowRunnerEvent>> {
    EVENT_SENDER
        .get()
        .cloned()
        .expect("Event sender not initialized")
}

pub fn request_redraw() {
    let Some(sender) = EVENT_SENDER.get() else {
        return;
    };
    if let Err(e) = sender.send_event(WindowRunnerEvent::Redraw()) {
        tracing::trace!("Failed to send Redraw event to window event loop: {}", e);
    }
    if let Err(e) = sender.send_event(WindowRunnerEvent::InvalidateSvelteState()) {
        tracing::trace!("Failed to send InvalidateSvelteState event to window event loop: {}", e);
    }
}

pub fn request_redraw_without_webview() {
    let Some(sender) = EVENT_SENDER.get() else {
        return;
    };
    if let Err(e) = sender.send_event(WindowRunnerEvent::Redraw()) {
        tracing::trace!("Failed to send Redraw event to window event loop: {}", e);
    }
}