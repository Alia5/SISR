use std::mem::discriminant;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use sdl3_sys::events::SDL_Event;

use crate::app::input::context::Context;
use crate::app::input::event::handler_events::InputHandlerEvent;
use crate::app::input::event::router::{EventHandler, ListenEvent, RoutedEvent};
use crate::app::input::sdl_loop::Subsystems;
use crate::app::steam::binding_enforcer::binding_enforcer;
use crate::config::CONFIG;

pub struct Handler {
    ctx: Arc<Mutex<Context>>,
    config_enforce_active_before: AtomicBool,
}

impl Handler {
    pub fn new(context: Arc<Mutex<Context>>) -> Self {
        Self {
            ctx: context,
            config_enforce_active_before: AtomicBool::new(false),
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
        let event = match event {
            Some(RoutedEvent::UserEvent(event)) => event,
            _ => {
                tracing::warn!("Received non-handler event ");
                return;
            }
        };
        let open = match event {
            InputHandlerEvent::OverlayStateChanged { open } => *open,
            _ => {
                tracing::warn!("Received non-OverlayStateChanged event ");
                return;
            }
        };

        let _continuous_draw_in_config = CONFIG
            .read()
            .ok()
            .and_then(|c| {
                c.as_ref()
                    .map(|cfg| cfg.window.continuous_draw.unwrap_or(false))
            })
            .unwrap_or(false);

        // TODO: FIXME: controller config enforcment revert and reset!
        // TODO: maybe has to be done earlier, like on guide-press...

        let Ok(mut ctx) = self.ctx.lock() else {
            tracing::error!("Failed to lock Context mutex");
            return;
        };
        if ctx.steam_overlay_open == open {
            tracing::debug!("Overlay state unchanged (still open: {})", open);
            return;
        }
        ctx.steam_overlay_open = open;
        if open {
            if let Ok(mut enforcer) = binding_enforcer().lock() {
                let active = enforcer.is_active();
                self.config_enforce_active_before
                    .store(active, std::sync::atomic::Ordering::Relaxed);
                if active {
                    tracing::debug!(
                        "Deactivating controller binding enforcer while overlay is open"
                    );
                    enforcer.deactivate();
                }
            }
        } else {
            if self
                .config_enforce_active_before
                .load(std::sync::atomic::Ordering::Relaxed)
            {
                tracing::debug!("Re-activating controller binding enforcer after overlay closed");
                if let Ok(mut enforcer) = binding_enforcer().lock() {
                    enforcer.activate();
                }
            }
        }
    }

    fn listen_events(&self) -> Vec<ListenEvent> {
        vec![ListenEvent::HandlerEvent(discriminant(
            &InputHandlerEvent::OverlayStateChanged { open: false },
        ))]
    }
}
