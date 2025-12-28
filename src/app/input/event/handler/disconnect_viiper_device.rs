use std::mem::discriminant;
use std::sync::{Arc, Mutex};

use sdl3_sys::events::SDL_Event;

use crate::app::input::context::Context;
use crate::app::input::event::handler_events::HandlerEvent;
use crate::app::input::event::router::{EventHandler, ListenEvent, RoutedEvent};
use crate::app::input::sdl_loop::Subsystems;
use crate::app::input::viiper_bridge::ViiperBridge;

pub struct Handler {
    ctx: Arc<Mutex<Context>>,
    viiper_bridge: Arc<Mutex<ViiperBridge>>,
}
impl Handler {
    pub fn new(ctx: Arc<Mutex<Context>>, viiper_bridge: Arc<Mutex<ViiperBridge>>) -> Self {
        Self { ctx, viiper_bridge }
    }
}

impl EventHandler for Handler {
    fn handle_event(
        &self,
        _subsystems: &Subsystems,
        event: &Option<RoutedEvent>,
        _sdl_event: &SDL_Event,
    ) {
        tracing::debug!(event = ?event);
        let event = match event {
            Some(RoutedEvent::UserEvent(event)) => event,
            _ => {
                tracing::warn!("Received non-handler event ");
                return;
            }
        };
        let device_id = match event {
            HandlerEvent::DisconnectViiperDevice { device_id } => *device_id,
            _ => {
                tracing::warn!("Received non-DisconnectViiperDevice event ");
                return;
            }
        };

        let Ok(ctx) = self.ctx.lock() else {
            tracing::error!("Failed to lock Context mutex");
            return;
        };
        let Some(device_mtx) = ctx.device_for_id(device_id) else {
            tracing::warn!("No device found for id {}", device_id);
            return;
        };
        drop(ctx);

        let Ok(mut device) = device_mtx.lock() else {
            tracing::error!("Failed to lock Device mutex for device id {}", device_id);
            return;
        };

        let Ok(mut viiper) = self.viiper_bridge.lock() else {
            tracing::error!("Failed to lock ViiperBridge mutex");
            return;
        };

        viiper.remove_device(device_id);
        device.viiper_device = None;
    }

    fn listen_events(&self) -> Vec<ListenEvent> {
        vec![ListenEvent::HandlerEvent(discriminant(
            &HandlerEvent::DisconnectViiperDevice { device_id: 0 },
        ))]
    }
}
