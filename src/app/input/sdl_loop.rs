use std::panic;
use std::sync::{Arc, OnceLock};

use crate::app::input::event::handler::{
    cef_debug_ready, connect_viiper_device, disconnect_viiper_device, ignore_device, kbm_key_event,
    kbm_pointer_event, kbm_release_all, overlay_state_changed, sdl_device_connected,
    sdl_device_disconnected, sdl_gamepad_steam_handle_updated, sdl_gamepad_update_complete,
    sdl_joy_device_removed, sdl_joystick_update_complete, set_kbm_emulation_enabled, viiper_ready,
};
use crate::app::input::event::handler_events::HandlerEvent;
use crate::app::input::event::router::EventRouter;
use crate::app::{App, input_old::sdl_hints};
use sdl3::event::EventSender;
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

#[allow(dead_code)]
struct Subsystems {
    joystick: JoystickSubsystem,
    gamepad: GamepadSubsystem,
    event: EventSubsystem,
}

pub struct InputLoop {
    #[allow(dead_code)]
    viiper_address: Option<std::net::SocketAddr>,
    #[allow(dead_code)]
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
            Err(_e) => {
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
        router.register_multiple(&[
            Arc::new(sdl_device_connected::Handler {}),
            Arc::new(sdl_device_disconnected::Handler {}),
            Arc::new(sdl_gamepad_steam_handle_updated::Handler {}),
            Arc::new(sdl_gamepad_update_complete::Handler {}),
            Arc::new(sdl_joystick_update_complete::Handler {}),
            Arc::new(sdl_joy_device_removed::Handler {}),
            Arc::new(ignore_device::Handler {}),
            Arc::new(connect_viiper_device::Handler {}),
            Arc::new(disconnect_viiper_device::Handler {}),
            Arc::new(cef_debug_ready::Handler {}),
            Arc::new(overlay_state_changed::Handler {}),
            Arc::new(set_kbm_emulation_enabled::Handler {}),
            Arc::new(kbm_key_event::Handler {}),
            Arc::new(kbm_pointer_event::Handler {}),
            Arc::new(kbm_release_all::Handler {}),
            Arc::new(viiper_ready::Handler {}),
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
