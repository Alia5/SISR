use sdl3_sys::events::{SDL_EVENT_GAMEPAD_UPDATE_COMPLETE, SDL_Event};

use crate::app::input::event::router::{EventHandler, ListenEvent, RoutedEvent};

pub struct Handler {}

impl EventHandler for Handler {
    fn handle_event(&self, _event: &Option<RoutedEvent>, _sdl_event: &SDL_Event) {}

    fn listen_events(&self) -> Vec<ListenEvent> {
        vec![ListenEvent::SdlEventType(SDL_EVENT_GAMEPAD_UPDATE_COMPLETE)]
    }
}
