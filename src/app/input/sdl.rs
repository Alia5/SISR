use std::sync::{Arc, Mutex};

use crate::app::{
    App,
    gui::dispatcher::GuiDispatcher,
    input::{handler::EventHandler, sdl_hints},
    window::RunnerEvent,
};
use sdl3::sys::events::{
    SDL_EVENT_GAMEPAD_STEAM_HANDLE_UPDATED, SDL_EVENT_GAMEPAD_UPDATE_COMPLETE,
    SDL_EVENT_JOYSTICK_UPDATE_COMPLETE, SDL_Event, SDL_PollEvent, SDL_WaitEvent,
};
use sdl3::{
    event::{Event, EventSender},
    gamepad::Gamepad,
};
use tracing::{Level, debug, error, span, trace, warn};
use winit::event_loop::EventLoopProxy;

pub fn get_gamepad_steam_handle(pad: &Gamepad) -> u64 {
    use sdl3::sys::gamepad::SDL_GetGamepadSteamHandle;
    let instance_id = pad.id().unwrap_or(0);
    if instance_id == 0 {
        trace!("Cannot get steam handle for device with invalid instance ID 0");
        return 0;
    }

    unsafe {
        // Extract the raw SDL_Gamepad pointer from the opened gamepad
        // sdl3-0.16.2\src\sdl3\gamepad.rs:745
        #[repr(C)]
        struct GamepadRaw {
            _subsystem: [u8; std::mem::size_of::<sdl3::GamepadSubsystem>()],
            raw: *mut sdl3::sys::gamepad::SDL_Gamepad,
        }

        let gamepad_raw: &GamepadRaw = std::mem::transmute(pad);
        if gamepad_raw.raw.is_null() {
            warn!(
                "Gamepad raw pointer is null for instance ID {}",
                instance_id
            );
            return 0;
        }

        SDL_GetGamepadSteamHandle(gamepad_raw.raw)
    }
}

#[macro_export]
macro_rules! event_which {
    ($event:expr) => {
        match $event {
            Event::JoyAxisMotion { which, .. }
            // | Event::JoyBallMotion { which, .. }
            | Event::JoyHatMotion { which, .. }
            | Event::JoyButtonDown { which, .. }
            | Event::JoyButtonUp { which, .. }
            | Event::JoyDeviceAdded { which, .. }
            | Event::JoyDeviceRemoved { which, .. }
            // FUCK RUSTFMT
            | Event::ControllerAxisMotion { which, .. }
            | Event::ControllerButtonDown { which, .. }
            | Event::ControllerButtonUp { which, .. }
            | Event::ControllerDeviceAdded { which, .. }
            | Event::ControllerDeviceRemoved { which, .. }
            | Event::ControllerDeviceRemapped { which, .. }
            | Event::ControllerTouchpadDown { which, .. }
            | Event::ControllerTouchpadMotion { which, .. }
            | Event::ControllerTouchpadUp { which, .. }
            | Event::ControllerSensorUpdated { which, .. } => Some(*which),
            _ => None,
        }
    };
}

pub struct InputLoop {
    sdl_waker: Arc<Mutex<Option<EventSender>>>,
    winit_waker: Arc<Mutex<Option<EventLoopProxy<RunnerEvent>>>>,
    gui_dispatcher: Arc<Mutex<Option<GuiDispatcher>>>,
    async_handle: tokio::runtime::Handle,
}

impl InputLoop {
    pub fn new(
        sdl_waker: Arc<Mutex<Option<EventSender>>>,
        winit_waker: Arc<Mutex<Option<EventLoopProxy<RunnerEvent>>>>,
        gui_dispatcher: Arc<Mutex<Option<GuiDispatcher>>>,
        async_handle: tokio::runtime::Handle,
    ) -> Self {
        Self {
            sdl_waker,
            winit_waker,
            gui_dispatcher,
            async_handle,
        }
    }

    pub fn run(&mut self, viiper_address: Option<std::net::SocketAddr>) {
        trace!("SDL_Init");

        for (hint_name, hint_value) in sdl_hints::SDL_HINTS {
            match sdl3::hint::set(hint_name, hint_value) {
                true => trace!("Set SDL hint {hint_name}={hint_value}"),
                false => warn!("Failed to set SDL hint {hint_name}={hint_value}"),
            }
        }
        let sdl = match sdl3::init() {
            Ok(sdl) => sdl,
            Err(e) => {
                error!("Failed to initialize SDL: {}", e);
                return;
            }
        };

        let joystick_subsystem = match sdl.joystick() {
            Ok(js) => js,
            Err(e) => {
                error!("Failed to initialize SDL joystick subsystem: {e}");
                return;
            }
        };
        let gamepad_subsystem = match sdl.gamepad() {
            Ok(gp) => gp,
            Err(e) => {
                error!("Failed to initialize SDL gamepad subsystem: {e}");
                return;
            }
        };

        let _events = match sdl.event() {
            Ok(event_subsystem) => {
                if let Err(e) =
                    event_subsystem.register_custom_event::<super::handler::HandlerEvent>()
                {
                    error!("Failed to register VIIPER disconnect event: {}", e);
                }

                match self.sdl_waker.lock() {
                    Ok(mut guard) => {
                        *guard = Some(event_subsystem.event_sender());
                    }
                    Err(e) => {
                        error!("Failed to set SDL event sender: {}", e);
                    }
                }
                event_subsystem
            }
            Err(e) => {
                error!("Failed to get SDL event subsystem: {}", e);
                return;
            }
        };

        let _event_pump = match sdl.event_pump() {
            Ok(pump) => pump,
            Err(e) => {
                error!("Failed to get SDL event pump: {}", e);
                return;
            }
        };

        if let Ok(dispatcher_guard) = self.gui_dispatcher.lock()
            && let Some(dispatcher) = &*dispatcher_guard
        {
            debug!("SDL loop GUI dispatcher initialized");
            dispatcher.register_callback(move |ctx| {
                InputLoop::on_draw(ctx);
            });
        }

        match self.run_loop(viiper_address, joystick_subsystem, gamepad_subsystem) {
            Ok(_) => {}
            Err(_) => {
                error!("SDL loop exited with error");
            }
        }

        trace!("SDL loop exiting");
        App::shutdown(None, Some(&self.winit_waker));
    }

    fn run_loop(
        &mut self,
        viiper_address: Option<std::net::SocketAddr>,
        joystick_subsystem: sdl3::JoystickSubsystem,
        gamepad_subsystem: sdl3::GamepadSubsystem,
    ) -> Result<(), ()> {
        let span = span!(Level::INFO, "sdl_loop");

        let mut pad_event_handler = EventHandler::new(
            self.sdl_waker.clone(),
            self.winit_waker.clone(),
            self.gui_dispatcher.clone(),
            viiper_address,
            self.async_handle.clone(),
            joystick_subsystem,
            gamepad_subsystem,
        );
        trace!("SDL loop starting");

        let mut sdl_event: SDL_Event = unsafe { std::mem::zeroed() };

        loop {
            let mut redraw = false;

            if !unsafe { SDL_WaitEvent(&mut sdl_event) } {
                continue;
            }
            if self.process_one(&mut sdl_event, &mut pad_event_handler, &span, &mut redraw)? {
                return Ok(());
            }
            while unsafe { SDL_PollEvent(&mut sdl_event) } {
                if self.process_one(&mut sdl_event, &mut pad_event_handler, &span, &mut redraw)? {
                    return Ok(());
                }
            }

            if redraw {
                self.request_redraw();
            }
        }
    }

    /// Returns Ok(true) to quit, Ok(false) to continue, Err(()) on fatal error
    fn process_one(
        &self,
        sdl_event: &mut SDL_Event,
        handler: &mut EventHandler,
        span: &tracing::span::Span,
        redraw: &mut bool,
    ) -> Result<bool, ()> {
        let et = unsafe { sdl_event.r#type };
        match sdl3::sys::events::SDL_EventType(et) {
            SDL_EVENT_GAMEPAD_STEAM_HANDLE_UPDATED => {
                let which = unsafe { sdl_event.gdevice.which };
                handler.on_steam_handle_updated(which);
                *redraw = true;
            }
            SDL_EVENT_GAMEPAD_UPDATE_COMPLETE => {
                let which = unsafe { sdl_event.gdevice.which };
                handler.on_update_complete(which);
                *redraw = true;
            }
            SDL_EVENT_JOYSTICK_UPDATE_COMPLETE => {
                let which = unsafe { sdl_event.jdevice.which };
                handler.on_update_complete(which);
                *redraw = true;
            }
            _ => {
                let event = Event::from_ll(*sdl_event);
                match event {
                    Event::Quit { .. } => {
                        tracing::event!(parent: span, Level::INFO, event = ?event, "Quit event received");
                        return Ok(true);
                    }
                    Event::JoyDeviceAdded { .. } | Event::ControllerDeviceAdded { .. } => {
                        handler.on_pad_added(&event);
                        *redraw = true;
                    }
                    Event::JoyDeviceRemoved { .. } | Event::ControllerDeviceRemoved { .. } => {
                        handler.on_pad_removed(&event);
                        *redraw = true;
                    }
                    _ => {
                        if event.is_user_event()
                            && let Some(handler_event) =
                                event.as_user_event_type::<super::handler::HandlerEvent>()
                        {
                            match handler_event {
                                super::handler::HandlerEvent::ViiperEvent(ve) => {
                                    handler.on_viiper_event(ve);
                                }
                                super::handler::HandlerEvent::IgnoreDeviceEvent { device_id } => {
                                    handler.ignore_device(device_id);
                                }
                            }
                            return Ok(false);
                        }

                        if event.is_joy() || event.is_controller() {
                            handler.on_pad_event(&event);
                        } else {
                            tracing::event!(parent: span, Level::TRACE, event = ?event, "SDL event");
                        }
                    }
                }
            }
        }

        Ok(false)
    }

    fn request_redraw(&self) {
        if let Ok(guard) = self.winit_waker.lock()
            && let Some(proxy) = &*guard
        {
            match proxy.send_event(RunnerEvent::Redraw()) {
                Ok(_) => {}
                Err(e) => {
                    warn!("Failed to request GUI redraw: {}", e);
                }
            }
        }
    }

    fn on_draw(ctx: &egui::Context) {}
}
