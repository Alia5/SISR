use std::sync::{Arc, OnceLock};

use crate::app::input_v2::event::handler_events::HandlerEvent;
use crate::app::input_v2::event::router::{EventRouter, RoutedEvent};
use crate::app::window;
use crate::app::{App, input::sdl_hints, window::RunnerEvent};
use sdl3::event::{Event, EventSender};
use sdl3::sys::events::{SDL_Event, SDL_PollEvent, SDL_WaitEvent};
use sdl3::{EventSubsystem, GamepadSubsystem, JoystickSubsystem};
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
        Self {
            viiper_address,
            subsystems: None,
            router: EventRouter::new(),
        }
    }

    pub fn run(&mut self) {
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
                tracing::error!("Failed to initialize SDL: {}", e);
                return;
            }
        };

        let joystick_subsystem = match sdl.joystick() {
            Ok(js) => js,
            Err(e) => {
                tracing::error!("Failed to initialize SDL joystick subsystem: {e}");
                return;
            }
        };
        let gamepad_subsystem = match sdl.gamepad() {
            Ok(gp) => gp,
            Err(e) => {
                tracing::error!("Failed to initialize SDL gamepad subsystem: {e}");
                return;
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
                tracing::error!("Failed to get SDL event subsystem: {}", e);
                return;
            }
        };

        let sdl_systems = Subsystems {
            joystick: joystick_subsystem,
            gamepad: gamepad_subsystem,
            event: events,
        };
        self.subsystems = Some(sdl_systems);

        let _event_pump = match sdl.event_pump() {
            Ok(pump) => pump,
            Err(e) => {
                tracing::error!("Failed to get SDL event pump: {}", e);
                return;
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

        match self.run_loop() {
            Ok(_) => {}
            Err(_) => {
                tracing::error!("SDL loop exited with error");
            }
        }

        tracing::trace!("SDL loop exiting");
        App::shutdown();
    }

    fn run_loop(&mut self) -> Result<(), ()> {
        let span = span!(Level::INFO, "sdl_loop");

        tracing::trace!("SDL loop starting");

        let mut sdl_event: SDL_Event = unsafe { std::mem::zeroed() };

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
    }

    fn process_one(
        &self,
        sdl_event: &mut SDL_Event,
        span: &tracing::span::Span,
    ) -> Result<bool, ()> {
        let event = Event::from_ll(*sdl_event);
        match event {
            Event::Quit { .. } => {
                tracing::event!(parent: span, Level::INFO, event = ?event, "Quit event received");
                return Ok(true);
            }
            Event::Unknown { .. } => {
                self.router.route(&None, sdl_event);
            }
            _ => {
                if event.is_joy() {
                    // ignore joysticks for now
                }
                if event.is_user_event()
                    && let Some(handler_event) = event.as_user_event_type::<HandlerEvent>()
                {
                    self.router
                        .route(&Some(RoutedEvent::UserEvent(handler_event)), sdl_event);
                } else {
                    self.router
                        .route(&Some(RoutedEvent::SdlEvent(event)), sdl_event);
                }
            }
        }
        Ok(false)
    }
}
