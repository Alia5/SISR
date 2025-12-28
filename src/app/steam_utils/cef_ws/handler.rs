use crate::app::steam_utils::cef_ws::handlers;
use crate::app::steam_utils::cef_ws::messages::CefMessage;
use crate::app::steam_utils::cef_ws::response_writer::ResponseWriter;

use super::messages::WsResponse;

#[derive(Default)]
pub struct Handler {}

impl Handler {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn handle(&self, message: CefMessage) -> WsResponse {
        let writer = ResponseWriter::new();

        match message {
            CefMessage::Ping => {
                handlers::ping::handle(&message, &writer);
            }
            CefMessage::OverlayStateChanged { .. } => {
                handlers::overlay_changed::handle(&message, &writer);
            }
        }

        let data = writer.take_response();
        WsResponse::success(data)
    }
}
