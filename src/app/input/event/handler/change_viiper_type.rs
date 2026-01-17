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
        let (device_id, new_type) = match event {
            HandlerEvent::ChangeViiperType {
                device_id,
                viiper_type,
            } => (device_id, viiper_type.clone()),
            _ => {
                tracing::warn!("Received non-ChangeViiperType event ");
                return;
            }
        };

        // TODO: Disconnect reconnect logic...

        let Ok(ctx) = self.ctx.lock() else {
            tracing::error!("Failed to lock Context mutex");
            return;
        };
        let Some(device_mtx) = ctx.device_for_id(*device_id) else {
            tracing::warn!("No device found for id {}", device_id);
            return;
        };
        drop(ctx);

        let Ok(mut device) = device_mtx.lock() else {
            tracing::error!("Failed to lock Device mutex for device id {}", device_id);
            return;
        };

        let mut should_reconnect = false;
        if device.viiper_device.is_some() {
            let Ok(mut viiper) = self.viiper_bridge.lock() else {
                tracing::error!("Failed to lock ViiperBridge mutex");
                return;
            };
            viiper.remove_device(*device_id);
            should_reconnect = true;
        }

        device.viiper_type = Some(new_type.clone());

        if should_reconnect {
            let Ok(viiper) = self.viiper_bridge.lock() else {
                tracing::error!("Failed to lock ViiperBridge mutex");
                return;
            };
            viiper.create_device(
                device.id,
                device.viiper_type.clone().unwrap_or(new_type).as_str(),
            );
        }

        //
    }

    fn listen_events(&self) -> Vec<ListenEvent> {
        vec![ListenEvent::HandlerEvent(discriminant(
            &HandlerEvent::ChangeViiperType {
                device_id: 0,
                viiper_type: String::new(),
            },
        ))]
    }
}
