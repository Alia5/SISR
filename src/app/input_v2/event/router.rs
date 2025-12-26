use std::collections::HashMap;

use sdl3::event::Event;
use sdl3_sys::events::{SDL_Event, SDL_EventType};

use crate::app::input_v2::event::handler_events::HandlerEvent;

pub enum RoutedEvent {
    SdlEvent(Event),
    UserEvent(HandlerEvent),
}

pub trait EventHandler {
    fn handle_event(&self, event: &Option<RoutedEvent>, sdl_event: &SDL_Event);
}

pub struct EventRouter {
    //
    handler_map: HashMap<SDL_EventType, Box<dyn EventHandler>>,
}

impl EventRouter {
    pub fn new() -> Self {
        Self {
            handler_map: HashMap::new(),
        }
    }

    pub fn register_handler<F>(&mut self, event_type: SDL_EventType, handler: F)
    where
        F: EventHandler + 'static,
    {
        self.handler_map.insert(event_type, Box::new(handler));
    }

    pub fn route(&self, event: &Option<RoutedEvent>, sdl_event: &SDL_Event) {
        let event_type = SDL_EventType(unsafe { sdl_event.r#type });
        let span = tracing::span!(tracing::Level::TRACE, "event", ?event_type);
        let _enter = span.enter();

        if let Some(handler) = self.handler_map.get(&event_type) {
            handler.handle_event(event, sdl_event);
        } else {
            // TODO!
            tracing::warn!("No handler registered for event type: {:?}", event_type);
        }
    }
}
