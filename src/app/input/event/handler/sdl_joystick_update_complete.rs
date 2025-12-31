use sdl3_sys::events::{SDL_EVENT_JOYSTICK_UPDATE_COMPLETE, SDL_Event, SDL_EventType};

use crate::app::input::event::router::{EventHandler, ListenEvent, RoutedEvent};
use crate::app::input::sdl_loop::Subsystems;

pub struct Handler {}

impl EventHandler for Handler {
    fn handle_event(
        &self,
        _subsystems: &Subsystems,
        _event: &Option<RoutedEvent>,
        _sdl_event: &SDL_Event,
    ) {
        // let event_type = SDL_EventType(unsafe { sdl_event.r#type });
        // tracing::trace!(event = ?event_type);
    }

    fn listen_events(&self) -> Vec<ListenEvent> {
        vec![ListenEvent::SdlEventType(
            SDL_EVENT_JOYSTICK_UPDATE_COMPLETE,
        )]
    }
}
