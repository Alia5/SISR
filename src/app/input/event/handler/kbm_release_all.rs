use std::mem::discriminant;
use std::sync::{Arc, Mutex};

use sdl3_sys::events::SDL_Event;
use viiper_client::devices::keyboard::KeyboardInput;
use viiper_client::devices::mouse::MouseInput;

use crate::app::input::context::Context;
use crate::app::input::event::handler_events::HandlerEvent;
use crate::app::input::event::kbm_context::KbmContext;
use crate::app::input::event::router::{EventHandler, ListenEvent, RoutedEvent};
use crate::app::input::sdl_loop::Subsystems;
use crate::app::input::viiper_bridge::ViiperBridge;
use crate::app::window::{self};

pub struct Handler {
    _ctx: Arc<Mutex<Context>>,
    viiper_bridge: Arc<Mutex<ViiperBridge>>,
    kbm_ctx: Arc<Mutex<KbmContext>>,
}
impl Handler {
    pub fn new(
        ctx: Arc<Mutex<Context>>,
        viiper_bridge: Arc<Mutex<ViiperBridge>>,
        kbm_ctx: Arc<Mutex<KbmContext>>,
    ) -> Self {
        Self {
            _ctx: ctx,
            viiper_bridge,
            kbm_ctx,
        }
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
        if !window::is_kbm_emulation_enabled() {
            return;
        }

        let event = match event {
            Some(RoutedEvent::UserEvent(event)) => event,
            _ => {
                tracing::warn!("Received non-handler event");
                return;
            }
        };
        match event {
            HandlerEvent::KbmReleaseAll() => {}
            _ => {
                tracing::warn!("Received non-KbmReleaseAll event");
                return;
            }
        };

        let Ok(mut kbm_ctx) = self.kbm_ctx.lock() else {
            tracing::error!("Failed to lock kbm_ctx");
            return;
        };
        kbm_ctx.keyboard_keys.clear();
        kbm_ctx.keyboard_modifiers = 0;
        kbm_ctx.mouse_buttons = 0;

        let Ok(viiper) = self.viiper_bridge.lock() else {
            tracing::error!("Failed to lock ViiperBridge");
            return;
        };

        if let Some(kbd_id) = kbm_ctx.keyboard_id {
            viiper.update_device_state(kbd_id, KeyboardInput::default());
        }
        if let Some(mouse_id) = kbm_ctx.mouse_id {
            viiper.update_device_state(mouse_id, MouseInput::default());
        }
        tracing::debug!("Released all KBM inputs");
    }

    fn listen_events(&self) -> Vec<ListenEvent> {
        vec![ListenEvent::HandlerEvent(discriminant(
            &HandlerEvent::KbmReleaseAll(),
        ))]
    }
}
