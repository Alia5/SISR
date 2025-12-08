use std::sync::{Arc, Mutex};
use tracing::{debug, warn};
use winit::event_loop::EventLoopProxy;

use crate::app::input::handler::HandlerEvent;
use crate::app::steam_utils::cef_ws::CefMessage;
use crate::app::steam_utils::cef_ws::response_writer::ResponseWriter;
use crate::app::window::RunnerEvent;

pub fn handle(
    message: &CefMessage,
    winit_waker: &Arc<Mutex<Option<EventLoopProxy<RunnerEvent>>>>,
    sdl_waker: &Arc<Mutex<Option<sdl3::event::EventSender>>>,
    _: &ResponseWriter,
) {
    let CefMessage::OverlayStateChanged { open } = message else {
        unreachable!("overlay_changed handler called with wrong message type");
    };

    debug!("CEF Debug WebSocket: Overlay state changed to: {}", open);

    if let Ok(guard) = winit_waker.lock() {
        if let Some(proxy) = &*guard {
            let _ = proxy.send_event(RunnerEvent::Redraw());
        }
    } else {
        warn!("Failed to acquire winit waker lock");
    }
    if let Ok(guard) = sdl_waker.lock()
        && let Some(sender) = &*guard
        && let Err(e) = sender.push_custom_event(HandlerEvent::OverlayStateChanged { open: *open })
    {
        warn!("Failed to push OverlayStateChanged event: {}", e);
    }
}
