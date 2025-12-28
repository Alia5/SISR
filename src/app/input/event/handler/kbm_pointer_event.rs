use std::mem::discriminant;
use std::sync::{Arc, Mutex};

use sdl3_sys::events::SDL_Event;
use viiper_client::devices::mouse::MouseInput;

use crate::app::input::context::Context;
use crate::app::input::event::handler_events::HandlerEvent;
use crate::app::input::event::kbm_context::KbmContext;
use crate::app::input::event::router::{EventHandler, ListenEvent, RoutedEvent};
use crate::app::input::kbm_events;
use crate::app::input::sdl_loop::Subsystems;
use crate::app::input::viiper_bridge::ViiperBridge;
use crate::app::window;

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
        tracing::trace!(event = ?event);
        let event = match event {
            Some(RoutedEvent::UserEvent(event)) => event,
            _ => {
                tracing::warn!("Received non-handler event ");
                return;
            }
        };
        let (dx, dy, wheel, pan, button, button_down) = match event {
            HandlerEvent::KbmPointerEvent(kbm_events::KbmPointerEvent {
                dx,
                dy,
                wheel_y,
                wheel_x,
                button,
                button_down,
            }) => {
                // TODO: find out IF we loose something and how much...
                // although sub-pixel mouse movements don't really make sense... eh..
                let dx = dx.round().clamp(i16::MIN as f32, i16::MAX as f32) as i16;
                let dy = dy.round().clamp(i16::MIN as f32, i16::MAX as f32) as i16;

                let wheel = wheel_y.round().clamp(i16::MIN as f32, i16::MAX as f32) as i16;
                let pan = wheel_x.round().clamp(i16::MIN as f32, i16::MAX as f32) as i16;
                (dx, dy, wheel, pan, *button, *button_down)
            }
            _ => {
                tracing::warn!("Received non-KbmPointerEvent event");
                return;
            }
        };
        if !window::is_kbm_emulation_enabled() {
            return;
        }
        let Ok(mut kbm_ctx) = self.kbm_ctx.lock() else {
            tracing::error!("Failed to lock kbm_ctx ");
            return;
        };

        let mouse_id = match kbm_ctx.mouse_id {
            Some(id) => id,
            None => {
                tracing::warn!("No virtual mouse device available");
                return;
            }
        };

        if button != 0 {
            let max_button = u8::BITS as u8;
            if button <= max_button {
                let shift = button - 1;
                if let Some(mask) = 1u8.checked_shl(shift as u32) {
                    if button_down {
                        kbm_ctx.mouse_buttons |= mask;
                    } else {
                        kbm_ctx.mouse_buttons &= !mask;
                    }
                }
            }
        }
        let buttons = kbm_ctx.mouse_buttons;
        drop(kbm_ctx);

        let Ok(viiper) = self.viiper_bridge.lock() else {
            tracing::error!("Failed to lock ViiperBridge");
            return;
        };
        viiper.update_device_state(
            mouse_id,
            MouseInput {
                dx,
                dy,
                buttons,
                wheel,
                pan,
            },
        );
    }

    fn listen_events(&self) -> Vec<ListenEvent> {
        vec![ListenEvent::HandlerEvent(discriminant(
            &HandlerEvent::KbmPointerEvent(kbm_events::KbmPointerEvent {
                dx: 0.0,
                dy: 0.0,
                wheel_y: 0.0,
                wheel_x: 0.0,
                button: 0,
                button_down: false,
            }),
        ))]
    }
}
