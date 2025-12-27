use tracing::{debug, warn};

use crate::app::input::event::handler_events::HandlerEvent;
use crate::app::input::sdl_loop;
use crate::app::steam_utils::cef_ws::CefMessage;
use crate::app::steam_utils::cef_ws::response_writer::ResponseWriter;
use crate::app::window::{self, RunnerEvent};

pub fn handle(message: &CefMessage, _: &ResponseWriter) {
    let CefMessage::OverlayStateChanged { open } = message else {
        unreachable!("overlay_changed handler called with wrong message type");
    };

    debug!("CEF Debug WebSocket: Overlay state changed to: {}", open);

    if let Err(e) = window::get_event_sender().send_event(RunnerEvent::Redraw()) {
        warn!("Failed to push Redraw event: {}", e);
    }
    if let Err(e) = sdl_loop::get_event_sender()
        .push_custom_event(HandlerEvent::OverlayStateChanged { open: *open })
    {
        warn!("Failed to push OverlayStateChanged event: {}", e);
    }
}
