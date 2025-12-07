use std::sync::{Arc, Mutex};
use tracing::{debug, warn};
use winit::event_loop::EventLoopProxy;

use crate::app::steam_utils::cef_ws::CefMessage;
use crate::app::steam_utils::cef_ws::messages::WsResponse;
use crate::app::window::RunnerEvent;

pub fn handle(
    message: &CefMessage,
    winit_waker: &Arc<Mutex<Option<EventLoopProxy<RunnerEvent>>>>,
    _sdl_waker: &Arc<Mutex<Option<sdl3::event::EventSender>>>,
) -> WsResponse {
    let CefMessage::OverlayStateChanged { open } = message else {
        unreachable!("overlay_changed handler called with wrong message type");
    };

    debug!("CEF Debug WebSocket: Overlay state changed to: {}", open);

    // TODO: Handle overlay state change
    // For now, we could potentially wake the winit event loop
    // or push a custom event if needed

    if let Ok(guard) = winit_waker.lock() {
        if let Some(proxy) = &*guard {
            let _ = proxy.send_event(RunnerEvent::Redraw());
        }
    } else {
        warn!("Failed to acquire winit waker lock");
    }

    WsResponse::success(None)
}
