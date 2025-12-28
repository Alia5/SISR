use std::mem::discriminant;
use std::sync::{Arc, Mutex};

use sdl3::keyboard::Scancode;
use sdl3_sys::events::SDL_Event;
use viiper_client::devices::keyboard::{KeyboardInput, constants as kb_const};

use crate::app::input::context::Context;
use crate::app::input::event::handler_events::HandlerEvent;
use crate::app::input::event::kbm_context::KbmContext;
use crate::app::input::event::router::{EventHandler, ListenEvent, RoutedEvent};
use crate::app::input::kbm_events;
use crate::app::input::sdl_loop::Subsystems;
use crate::app::input::viiper_bridge::ViiperBridge;
use crate::app::window;

pub struct Handler {
    ctx: Arc<Mutex<Context>>,
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
            ctx,
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
        let (scancode, down) = match event {
            HandlerEvent::KbmKeyEvent(kbm_events::KbmKeyEvent { scancode, down }) => {
                (*scancode, *down)
            }
            _ => {
                tracing::warn!("Received non-KbmKeyEvent event ");
                return;
            }
        };
        if !window::is_kbm_emulation_enabled() {
            return;
        }
        let Ok(ctx) = self.ctx.lock() else {
            tracing::error!("Failed to lock context ");
            return;
        };
        let Ok(mut kbm_ctx) = self.kbm_ctx.lock() else {
            tracing::error!("Failed to lock kbm_ctx ");
            return;
        };
        let Some(kbd_id) = kbm_ctx.keyboard_id else {
            tracing::warn!("No keyboard device found ");
            return;
        };
        drop(ctx);

        let modifier_bit = match scancode {
            x if x == Scancode::LCtrl as u16 => Some(kb_const::MOD_LEFT_CTRL),
            x if x == Scancode::LShift as u16 => Some(kb_const::MOD_LEFT_SHIFT),
            x if x == Scancode::LAlt as u16 => Some(kb_const::MOD_LEFT_ALT),
            x if x == Scancode::LGui as u16 => Some(kb_const::MOD_LEFT_GUI),
            x if x == Scancode::RCtrl as u16 => Some(kb_const::MOD_RIGHT_CTRL),
            x if x == Scancode::RShift as u16 => Some(kb_const::MOD_RIGHT_SHIFT),
            x if x == Scancode::RAlt as u16 => Some(kb_const::MOD_RIGHT_ALT),
            x if x == Scancode::RGui as u16 => Some(kb_const::MOD_RIGHT_GUI),
            _ => None,
        };

        if let Some(bit) = modifier_bit {
            if down {
                kbm_ctx.keyboard_modifiers |= bit;
            } else {
                kbm_ctx.keyboard_modifiers &= !bit;
            }
        } else {
            let Ok(key) = u8::try_from(scancode) else {
                tracing::warn!("KBM scancode out of range ({}); dropping", scancode);
                return;
            };
            if down {
                _ = kbm_ctx.keyboard_keys.insert(key);
            } else {
                _ = kbm_ctx.keyboard_keys.remove(&key);
            }
        }
        let modifiers = kbm_ctx.keyboard_modifiers;
        let keys: Vec<u8> = kbm_ctx.keyboard_keys.iter().copied().collect();
        let count = u8::try_from(keys.len()).unwrap_or(u8::MAX);
        drop(kbm_ctx);

        let Ok(viiper) = self.viiper_bridge.lock() else {
            tracing::error!("Failed to lock ViiperBridge ");
            return;
        };
        viiper.update_device_state(
            kbd_id,
            KeyboardInput {
                modifiers,
                keys,
                count,
            },
        );
    }

    fn listen_events(&self) -> Vec<ListenEvent> {
        vec![ListenEvent::HandlerEvent(discriminant(
            &HandlerEvent::KbmKeyEvent(kbm_events::KbmKeyEvent {
                scancode: 0,
                down: false,
            }),
        ))]
    }
}
