use std::mem::discriminant;
use std::sync::{Arc, Mutex};

use sdl3_sys::events::SDL_Event;

use crate::app::input::context::Context;
use crate::app::input::event::handler_events::HandlerEvent;
use crate::app::input::event::router::{EventHandler, ListenEvent, RoutedEvent};
use crate::app::input::sdl_loop::{self, Subsystems};
use crate::app::input::viiper_bridge::ViiperBridge;
use crate::app::window;

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
        let version = match event {
            HandlerEvent::ViiperReady { version } => version,
            _ => {
                tracing::warn!("Received non-ViiperReady event ");
                return;
            }
        };

        let Ok(mut ctx) = self.ctx.lock() else {
            tracing::error!("Failed to lock Context mutex");
            return;
        };

        ctx.viiper_available = true;
        ctx.viiper_version = Some(version.clone());
        if ctx.keyboard_mouse_emulation {
            tracing::info!("Enabling keyboard/mouse emulation due to Viiper being ready");
            _ = sdl_loop::get_event_sender().push_custom_event(HandlerEvent::SetKbmEmulation {
                enabled: true,
                initialize: true,
            });
        }
        drop(ctx);

        let Ok(mut viiper) = self.viiper_bridge.lock() else {
            tracing::error!("Failed to lock ViiperBridge mutex");
            return;
        };

        viiper.set_ready(version);

        window::request_redraw();
    }

    fn listen_events(&self) -> Vec<ListenEvent> {
        vec![ListenEvent::HandlerEvent(discriminant(
            &HandlerEvent::ViiperReady {
                version: String::new(),
            },
        ))]
    }
}
