use std::mem::discriminant;

use sdl3::event::Event;
use sdl3_sys::events::SDL_Event;

use crate::app::input::event::router::{EventHandler, ListenEvent, RoutedEvent};

pub struct Handler {}

impl EventHandler for Handler {
    fn handle_event(&self, _event: &Option<RoutedEvent>, sdl_event: &SDL_Event) {
        let which = unsafe { sdl_event.gdevice.which };
        tracing::info!(id = which);
    }

    fn listen_events(&self) -> Vec<ListenEvent> {
        vec![ListenEvent::SdlEvent(discriminant(
            &Event::ControllerDeviceAdded {
                timestamp: 0,
                which: 0,
            },
        ))]
    }
}
