use std::mem::discriminant;
use std::sync::{Arc, Mutex};

use sdl3_sys::events::SDL_Event;

use crate::app::input::context::Context;
use crate::app::input::event::handler_events::HandlerEvent;
use crate::app::input::event::router::{EventHandler, ListenEvent, RoutedEvent};
use crate::app::input::sdl_loop::Subsystems;
use crate::app::window;
use crate::config::CONFIG;

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
        let open = match event {
            HandlerEvent::OverlayStateChanged { open } => *open,
            _ => {
                tracing::warn!("Received non-OverlayStateChanged event ");
                return;
            }
        };

        let continous_draw_in_config = CONFIG
            .read()
            .ok()
            .and_then(|c| {
                c.as_ref()
                    .map(|cfg| cfg.window.continous_draw.unwrap_or(false))
            })
            .unwrap_or(false);

        // TODO: FIXME: controller config enforcment revert and reset!
        // TODO: maybe has to be done earlier, like on guide-press...

        let Ok(mut ctx) = self.ctx.lock() else {
            tracing::error!("Failed to lock Context mutex");
            return;
        };
        ctx.steam_overlay_open = open;
        if open {
            window::set_continuous_redraw(true);
        } else {
            if continous_draw_in_config {
                return;
            }
            let cont_draw = window::is_continuous_redraw();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(1000));
                window::set_continuous_redraw(cont_draw);
            });
        }
    }

    fn listen_events(&self) -> Vec<ListenEvent> {
        vec![ListenEvent::HandlerEvent(discriminant(
            &HandlerEvent::OverlayStateChanged { open: false },
        ))]
    }
}
