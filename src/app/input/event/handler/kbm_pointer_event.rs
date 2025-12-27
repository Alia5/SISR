use std::mem::discriminant;

use sdl3_sys::events::SDL_Event;

use crate::app::input::event::handler_events::HandlerEvent;
use crate::app::input::event::router::{EventHandler, ListenEvent, RoutedEvent};
use crate::app::input_old::kbm_events;

pub struct Handler {}

impl EventHandler for Handler {
    fn handle_event(&self, _event: &Option<RoutedEvent>, _sdl_event: &SDL_Event) {}

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
