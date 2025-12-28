use std::panic;
use std::sync::{Arc, Mutex, OnceLock};

use crate::app::App;
use crate::app::gui::dispatcher::GuiDispatcher;
use crate::app::input::context::Context;
use crate::app::input::event::handler::{
    cef_debug_ready, connect_viiper_device, disconnect_viiper_device, ignore_device, kbm_key_event, kbm_pointer_event, kbm_release_all, on_controller_button_down, on_viiper_event, overlay_state_changed, sdl_device_connected, sdl_device_disconnected, sdl_gamepad_steam_handle_updated, sdl_gamepad_update_complete, sdl_steamdeck_updates, set_kbm_emulation, viiper_ready
};
use crate::app::input::event::handler_events::HandlerEvent;
use crate::app::input::event::kbm_context::KbmContext;
use crate::app::input::event::router::EventRouter;
use crate::app::input::gui::bottom_bar::BottomBar;
use crate::app::input::sdl_hints;
use crate::app::input::viiper_bridge::ViiperBridge;
use sdl3::event::EventSender;
use sdl3::sys::events::{SDL_Event, SDL_PollEvent, SDL_WaitEvent};
use sdl3::{EventSubsystem, GamepadSubsystem, JoystickSubsystem, SensorSubsystem};
use sdl3_sys::events::SDL_EventType;
use tracing::{Level, span};

static EVENT_SENDER: OnceLock<Arc<EventSender>> = OnceLock::new();

pub fn get_event_sender() -> Arc<EventSender> {
    EVENT_SENDER
        .get()
        .cloned()
        .expect("get sdl event sender called before initialized")
}

pub struct Subsystems {
    pub joystick: JoystickSubsystem,
    pub gamepad: GamepadSubsystem,
    pub sensor: SensorSubsystem,
    pub event: EventSubsystem,
}

pub struct InputLoop {
    subsystems: Subsystems,
    router: EventRouter,
    _viiper_bridge: Arc<Mutex<ViiperBridge>>,
    _context: Arc<Mutex<Context>>,
    _gui_dispatcher: Arc<Mutex<GuiDispatcher>>,
}

impl InputLoop {
    pub fn new(
        viiper_address: Option<std::net::SocketAddr>,
        gui_dispatcher: Arc<Mutex<GuiDispatcher>>,
    ) -> Self {
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

        let sensor_subsystem = match sdl.sensor() {
            Ok(s) => s,
            Err(e) => {
                panic!("Failed to initialize SDL sensor subsystem: {e}");
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
            sensor: sensor_subsystem,
            event: events,
        };

        let _event_pump = match sdl.event_pump() {
            Ok(pump) => pump,
            Err(e) => {
                panic!("Failed to get SDL event pump: {e}");
            }
        };
        let viiper_bridge = Arc::new(Mutex::new(ViiperBridge::new(viiper_address)));
        let context = Arc::new(Mutex::new(Context::new(viiper_address)));

        let kbm_context = Arc::new(Mutex::new(KbmContext::default()));

        tracing::debug!("Registering SDL event handlers");
        let mut router = EventRouter::default();
        router.register_multiple(&[
            Arc::new(on_controller_button_down::Handler::new(context.clone())),
            Arc::new(sdl_gamepad_update_complete::Handler::new(
                context.clone(),
                viiper_bridge.clone(),
            )),
            Arc::new(sdl_steamdeck_updates::Handler::new(
                context.clone(),
                viiper_bridge.clone(),
            )),
            Arc::new(sdl_device_connected::Handler::new(
                context.clone(),
                viiper_bridge.clone(),
            )),
            Arc::new(sdl_device_disconnected::Handler::new(
                context.clone(),
                viiper_bridge.clone(),
            )),
            Arc::new(on_viiper_event::Handler::new(
                context.clone(),
                viiper_bridge.clone(),
            )),
            Arc::new(sdl_gamepad_steam_handle_updated::Handler::new(
                context.clone(),
                viiper_bridge.clone(),
            )),
            // Arc::new(sdl_joystick_update_complete::Handler {}), // TODO:
            Arc::new(ignore_device::Handler::new(context.clone())),
            Arc::new(connect_viiper_device::Handler::new(
                context.clone(),
                viiper_bridge.clone(),
            )),
            Arc::new(disconnect_viiper_device::Handler::new(
                context.clone(),
                viiper_bridge.clone(),
            )),
            Arc::new(cef_debug_ready::Handler {}),
            Arc::new(overlay_state_changed::Handler::new(context.clone())),
            Arc::new(set_kbm_emulation::Handler::new(
                context.clone(),
                viiper_bridge.clone(),
                kbm_context.clone(),
            )),
            Arc::new(kbm_key_event::Handler::new(
                context.clone(),
                viiper_bridge.clone(),
                kbm_context.clone(),
            )),
            Arc::new(kbm_pointer_event::Handler::new(
                context.clone(),
                viiper_bridge.clone(),
                kbm_context.clone(),
            )),
            Arc::new(kbm_release_all::Handler::new(
                context.clone(),
                viiper_bridge.clone(),
                kbm_context.clone(),
            )),
            Arc::new(viiper_ready::Handler::new(
                context.clone(),
                viiper_bridge.clone(),
            )),
        ]);

        tracing::debug!("Registering GUI callbacks");
        let Ok(dispatcher) = gui_dispatcher.lock() else {
            panic!("Failed to lock GUI dispatcher mutex");
        };
        let uictx = context.clone();
        let bottom_bar = Arc::new(Mutex::new(BottomBar::new()));
        dispatcher.register_callback(move |ectx| {
            let Ok(ctx) = uictx.lock() else {
                tracing::error!("Failed to lock Context mutex for GUI drawing");
                return;
            };
            let Ok(mut bb) = bottom_bar.lock() else {
                tracing::error!("Failed to lock BottomBar mutex for GUI drawing");
                return;
            };
            bb.draw(&ctx, ectx);
        });
        drop(dispatcher);

        Self {
            subsystems: sdl_systems,
            router,
            _context: context,
            _viiper_bridge: viiper_bridge,
            _gui_dispatcher: gui_dispatcher,
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

        self.router.route(&self.subsystems, sdl_event);
        Ok(false)
    }
}
