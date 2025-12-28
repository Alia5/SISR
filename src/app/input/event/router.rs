use std::{collections::HashMap, mem::Discriminant, sync::Arc};

use sdl3::event::Event;
use sdl3_sys::events::{SDL_Event, SDL_EventType};

use crate::app::input::{event::handler_events::HandlerEvent, sdl_loop::Subsystems};

/// "Variadic" helper for registering multiple handlers.
///
/// Rust doesn't have variadic functions, so this macro expands to multiple
/// `register(...)` calls. It supports different concrete handler types.
///
/// Usage:
/// `event_router_register!(router, Arc::new(A {}), Arc::new(B {}));`
#[macro_export]
macro_rules! event_router_register {
    ($router:expr, $($handler:expr),+ $(,)?) => {{
        $(
            $router.register($handler);
        )+
    }};
}

#[derive(Debug)]
pub enum RoutedEvent {
    SdlEvent(Event),
    UserEvent(HandlerEvent),
}

pub enum ListenEvent {
    SdlEventType(SDL_EventType),
    SdlEvent(Discriminant<Event>),
    HandlerEvent(Discriminant<HandlerEvent>),
}

pub trait EventHandler {
    fn handle_event(
        &self,
        subsystems: &Subsystems,
        event: &Option<RoutedEvent>,
        sdl_event: &SDL_Event,
    );

    fn listen_events(&self) -> Vec<ListenEvent>;
}

#[derive(Default)]
pub struct EventRouter {
    //
    sdl_type_handler_map: HashMap<SDL_EventType, Arc<dyn EventHandler>>,
    sdl_event_handler_map: HashMap<Discriminant<Event>, Arc<dyn EventHandler>>,
    handler_event_handler_map: HashMap<Discriminant<HandlerEvent>, Arc<dyn EventHandler>>,
}

impl EventRouter {
    pub fn new() -> Self {
        Self {
            sdl_type_handler_map: HashMap::new(),
            sdl_event_handler_map: HashMap::new(),
            handler_event_handler_map: HashMap::new(),
        }
    }

    pub fn register_multiple(&mut self, handlers: &[Arc<dyn EventHandler>]) {
        for handler in handlers {
            self.register(handler.clone());
        }
    }

    pub fn register(&mut self, handler: Arc<dyn EventHandler>) {
        for listen_event in handler.listen_events() {
            match listen_event {
                ListenEvent::SdlEventType(event_type) => {
                    self.sdl_type_handler_map
                        .insert(event_type, handler.clone());
                }
                ListenEvent::SdlEvent(discriminant) => {
                    self.sdl_event_handler_map
                        .insert(discriminant, handler.clone());
                }
                ListenEvent::HandlerEvent(discriminant) => {
                    self.handler_event_handler_map
                        .insert(discriminant, handler.clone());
                }
            }
        }
    }

    pub fn route(&self, subsystems: &Subsystems, sdl_event: &SDL_Event) {
        let event_type = SDL_EventType(unsafe { sdl_event.r#type });
        let span = tracing::span!(tracing::Level::TRACE, "sdl_event", ?event_type);
        let _enter = span.enter();

        // tracing::trace!("Routing SDL event: {:?}", event_type);

        let hl_event = Event::from_ll(*sdl_event);
        if let Some(handler) = self.sdl_type_handler_map.get(&event_type) {
            handler.handle_event(
                subsystems,
                &Some(RoutedEvent::SdlEvent(hl_event)),
                sdl_event,
            );
            return;
        }
        let discriminant = std::mem::discriminant(&hl_event);
        if hl_event.is_user_event()
            && let Some(handler_event) = hl_event.as_user_event_type::<HandlerEvent>()
        {
            let handler_event_discriminant = std::mem::discriminant(&handler_event);
            if let Some(handler) = self
                .handler_event_handler_map
                .get(&handler_event_discriminant)
            {
                handler.handle_event(
                    subsystems,
                    &Some(RoutedEvent::UserEvent(handler_event)),
                    sdl_event,
                );
                return;
            }
            tracing::warn!(
                "No handler registered for handler event: {:?}",
                handler_event_discriminant
            );
            return;
        }
        if let Some(handler) = self.sdl_event_handler_map.get(&discriminant) {
            handler.handle_event(
                subsystems,
                &Some(RoutedEvent::SdlEvent(hl_event)),
                sdl_event,
            );
            return;
        }

        if hl_event.is_joy() || event_type == SDL_EventType::JOYSTICK_UPDATE_COMPLETE || event_type == SDL_EventType::GAMEPAD_AXIS_MOTION {
            // reduce log spam
            return;
        }

        tracing::trace!("No handler registered for event: {:?}", event_type);
    }
}
