use std::panic;
use std::sync::{Arc, OnceLock};

use crate::app::input::event::handler::{self, sdl_device_connected, sdl_device_disconnected};
use crate::app::input::event::handler_events::HandlerEvent;
use crate::app::input::event::router::{EventRouter, RoutedEvent};
use crate::app::window;
use crate::app::{App, input_old::sdl_hints, window::RunnerEvent};
use sdl3::event::{Event, EventSender};
use sdl3::sys::events::{SDL_Event, SDL_PollEvent, SDL_WaitEvent};
use sdl3::{EventSubsystem, GamepadSubsystem, JoystickSubsystem};
use sdl3_sys::events::SDL_EventType;
use tracing::{Level, span};

static EVENT_SENDER: OnceLock<Arc<EventSender>> = OnceLock::new();

pub fn get_event_sender() -> Arc<EventSender> {
    EVENT_SENDER
        .get()
        .cloned()
        .expect("get sdl event sender called before initialized")
}

struct Subsystems {
    joystick: JoystickSubsystem,
    gamepad: GamepadSubsystem,
    event: EventSubsystem,
}

pub struct InputLoop {
    viiper_address: Option<std::net::SocketAddr>,
    subsystems: Option<Subsystems>,
    router: EventRouter,
}

impl InputLoop {
    pub fn new(viiper_address: Option<std::net::SocketAddr>) -> Self {
        tracing::trace!("SDL_Init");

        for (hint_name, hint_value) in sdl_hints::SDL_HINTS {
            match sdl3::hint::set(hint_name, hint_value) {
                true => tracing::trace!("Set SDL hint {hint_name}={hint_value}"),
                false => tracing::warn!("Failed to set SDL hint {hint_name}={hint_value}"),
            }
        }
        let sdl = match sdl3::init() {
            Ok(sdl) => sdl,
            Err(e) => {
                panic!("Failed to initialize SDL");
            }
        };

        let joystick_subsystem = match sdl.joystick() {
            Ok(js) => js,
            Err(e) => {
                panic!("Failed to initialize SDL joystick subsystem: {e}");
            }
        };
        let gamepad_subsystem = match sdl.gamepad() {
            Ok(gp) => gp,
            Err(e) => {
                panic!("Failed to initialize SDL gamepad subsystem: {e}");
            }
        };

        let events = match sdl.event() {
            Ok(event_subsystem) => {
                if let Err(e) = event_subsystem.register_custom_event::<HandlerEvent>() {
                    tracing::error!("Failed to register VIIPER disconnect event: {}", e);
                }

                EVENT_SENDER
                    .set(Arc::new(event_subsystem.event_sender()))
                    .ok();

                event_subsystem
            }
            Err(e) => {
                panic!("Failed to initialize SDL event subsystem: {e}");
            }
        };

        let sdl_systems = Subsystems {
            joystick: joystick_subsystem,
            gamepad: gamepad_subsystem,
            event: events,
        };

        let _event_pump = match sdl.event_pump() {
            Ok(pump) => pump,
            Err(e) => {
                panic!("Failed to get SDL event pump: {e}");
            }
        };

        // if let Ok(dispatcher_guard) = self.gui_dispatcher.lock()
        //     && let Some(dispatcher) = &*dispatcher_guard
        // {
        //     debug!("SDL loop GUI dispatcher initialized");
        //     dispatcher.register_callback(move |ctx| {
        //         InputLoop::on_draw(ctx);
        //     });
        // }

        let mut router = EventRouter::default();
        // router.register(Arc::new(sdl_device_connected::handler {}));
        // router.register(Arc::new(sdl_device_disconnected::handler {}));
        router.register_multiple(&[
            Arc::new(sdl_device_connected::Handler {}),
            Arc::new(sdl_device_disconnected::Handler {}),
        ]);

        Self {
            viiper_address,
            subsystems: Some(sdl_systems),
            router,
        }
    }

    pub fn run(&mut self) {
        let span = span!(Level::INFO, "sdl_loop");

        tracing::trace!("SDL loop starting");

        let mut sdl_event: SDL_Event = unsafe { std::mem::zeroed() };

        match (|| -> Result<(), ()> {
            loop {
                if !unsafe { SDL_WaitEvent(&mut sdl_event) } {
                    continue;
                }
                if self.process_one(&mut sdl_event, &span)? {
                    return Ok(());
                }
                while unsafe { SDL_PollEvent(&mut sdl_event) } {
                    if self.process_one(&mut sdl_event, &span)? {
                        return Ok(());
                    }
                }
            }
        })() {
            Ok(_) => {}
            Err(_) => {
                tracing::error!("SDL loop encountered an error and is exiting");
            }
        }
        tracing::trace!("SDL loop exiting");
        App::shutdown();
    }

    fn process_one(
        &self,
        sdl_event: &mut SDL_Event,
        span: &tracing::span::Span,
    ) -> Result<bool, ()> {
        if unsafe { sdl_event.r#type } == SDL_EventType::QUIT.0 {
            tracing::event!(parent: span, Level::INFO, "Quit event received from window runner");
            return Ok(true);
        }

        self.router.route(sdl_event);
        Ok(false)
    }
}
