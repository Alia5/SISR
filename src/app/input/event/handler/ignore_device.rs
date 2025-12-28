use std::mem::discriminant;
use std::sync::{Arc, Mutex};

use sdl3_sys::events::SDL_Event;

use crate::app::input::context::Context;
use crate::app::input::event::handler_events::HandlerEvent;
use crate::app::input::event::router::{EventHandler, ListenEvent, RoutedEvent};
use crate::app::input::sdl_loop::{self, Subsystems};

pub struct Handler {
    ctx: Arc<Mutex<Context>>,
}

impl Handler {
    pub fn new(context: Arc<Mutex<Context>>) -> Self {
        Self { ctx: context }
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
            HandlerEvent::IgnoreDevice { device_id } => *device_id,
            _ => {
                tracing::warn!("Received non-IgnoreDevice event ");
                return;
            }
        };

        let Ok(ctx) = self.ctx.lock() else {
            tracing::error!("Failed to lock Context mutex");
            return;
        };
        let Some(device) = ctx.devices.iter().find(|r| {
            let Ok(d) = r.value().lock() else {
                tracing::error!("Failed to lock device mutex");
                return false;
            };
            d.id == device_id
        }) else {
            tracing::warn!("No device found with id {}", device_id);
            return;
        };
        let Ok(d) = device.value().lock() else {
            tracing::error!("Failed to lock device mutex");
            return;
        };
        let sender = sdl_loop::get_event_sender();
        for sdl_d in &d.sdl_devices {
            if sdl_d.gamepad.is_some() {
                _ = sender.push_event(sdl3::event::Event::ControllerDeviceRemoved {
                    timestamp: 0,
                    which: sdl_d.id,
                });
            }
            if sdl_d.joystick.is_some() {
                _ = sender.push_event(sdl3::event::Event::JoyDeviceRemoved {
                    timestamp: 0,
                    which: sdl_d.id,
                });
            }
        }

        tracing::info!("Ignoring device with id {}", device_id);
    }

    fn listen_events(&self) -> Vec<ListenEvent> {
        vec![ListenEvent::HandlerEvent(discriminant(
            &HandlerEvent::IgnoreDevice { device_id: 0 },
        ))]
    }
}
