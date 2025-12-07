use std::sync::{Arc, Mutex};
use winit::event_loop::EventLoopProxy;

use crate::app::steam_utils::cef_ws::handlers;
use crate::app::steam_utils::cef_ws::messages::CefMessage;
use crate::app::window::RunnerEvent;

use super::messages::WsResponse;

pub struct Handler {
    winit_waker: Arc<Mutex<Option<EventLoopProxy<RunnerEvent>>>>,
    sdl_waker: Arc<Mutex<Option<sdl3::event::EventSender>>>,
}

impl Handler {
    pub fn new(
        winit_waker: Arc<Mutex<Option<EventLoopProxy<RunnerEvent>>>>,
        sdl_waker: Arc<Mutex<Option<sdl3::event::EventSender>>>,
    ) -> Self {
        Self {
            winit_waker,
            sdl_waker,
        }
    }
    pub fn handle(&self, message: CefMessage) -> WsResponse {
        match message {
            CefMessage::Ping => {
                handlers::ping::handle(&message, &self.winit_waker, &self.sdl_waker)
            }
            CefMessage::OverlayStateChanged { .. } => {
                handlers::overlay_changed::handle(&message, &self.winit_waker, &self.sdl_waker)
            }
        }
    }
}
