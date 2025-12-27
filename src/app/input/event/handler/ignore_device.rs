use std::mem::discriminant;

use sdl3_sys::events::SDL_Event;

use crate::app::input::event::handler_events::HandlerEvent;
use crate::app::input::event::router::{EventHandler, ListenEvent, RoutedEvent};

pub struct Handler {}

impl EventHandler for Handler {
    fn handle_event(&self, _event: &Option<RoutedEvent>, _sdl_event: &SDL_Event) {}

    fn listen_events(&self) -> Vec<ListenEvent> {
        vec![ListenEvent::HandlerEvent(discriminant(
            &HandlerEvent::IgnoreDevice { device_id: 0 },
        ))]
    }
}
