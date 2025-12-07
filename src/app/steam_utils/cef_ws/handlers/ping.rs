use serde_json::json;
use std::sync::{Arc, Mutex};
use tracing::debug;
use winit::event_loop::EventLoopProxy;

use crate::app::steam_utils::cef_ws::CefMessage;
use crate::app::steam_utils::cef_ws::messages::WsResponse;
use crate::app::window::RunnerEvent;

pub fn handle(
    _message: &CefMessage,
    _winit_waker: &Arc<Mutex<Option<EventLoopProxy<RunnerEvent>>>>,
    _sdl_waker: &Arc<Mutex<Option<sdl3::event::EventSender>>>,
) -> WsResponse {
    debug!("CEF Debug WebSocket: Received ping");
    WsResponse::success(Some(json!({ "pong": true })))
}
