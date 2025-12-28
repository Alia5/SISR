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
            HandlerEvent::ConnectViiperDevice { device_id } => *device_id,
            _ => {
                tracing::warn!("Received non-ConnectViiperDevice event ");
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

        let Ok(device) = device_mtx.lock() else {
            tracing::error!("Failed to lock Device mutex for device id {}", device_id);
            return;
        };

        if device.viiper_device.is_some() {
            tracing::warn!(
                "Device id {} is already connected to a VIIPER device",
                device_id
            );
            return;
        }
        if device.steam_handle == 0 {
            tracing::warn!(
                "Device id {} has no Steam handle; cannot connect to VIIPER device",
                device_id
            );
            return;
        }

        let Ok(viiper) = self.viiper_bridge.lock() else {
            tracing::error!("Failed to lock ViiperBridge mutex");
            return;
        };
        viiper.create_device(
            device.id,
            device
                .viiper_type
                .clone()
                .unwrap_or("xbox360".to_string())
                .as_str(),
        );
    }

    fn listen_events(&self) -> Vec<ListenEvent> {
        vec![ListenEvent::HandlerEvent(discriminant(
            &HandlerEvent::ConnectViiperDevice { device_id: 0 },
        ))]
    }
}
