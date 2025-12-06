mod events_connection;
mod events_input;
mod events_misc;
mod gui;
mod viiper_bridge;

mod events;
pub use events::HandlerEvent;
use viiper_bridge::ViiperBridge;
pub use viiper_bridge::ViiperEvent;

use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use sdl3::event::EventSender;
use tracing::{debug, warn};
use winit::event_loop::EventLoopProxy;

use crate::app::{
    gui::dispatcher::GuiDispatcher,
    input::device::{Device, DeviceState, SDLDevice},
    window::RunnerEvent,
};
use crate::app::{
    input::handler::gui::bottom_bar::BottomBar, steam_utils::binding_enforcer::BindingEnforcer,
};

pub struct EventHandler {
    winit_waker: Arc<Mutex<Option<EventLoopProxy<RunnerEvent>>>>,
    gui_dispatcher: Arc<Mutex<Option<GuiDispatcher>>>,
    async_handle: tokio::runtime::Handle,
    sdl_joystick: sdl3::JoystickSubsystem,
    sdl_gamepad: sdl3::GamepadSubsystem,
    sdl_devices: HashMap<u32, Vec<SDLDevice>>,
    sdl_id_to_device: HashMap<u32, (u64, DeviceState)>,
    next_device_id: u64,
    viiper: ViiperBridge,
    state: Arc<Mutex<State>>,
}

pub(super) struct State {
    devices: HashMap<u64, Device>,
    viiper_address: Option<SocketAddr>,
    binding_enforcer: BindingEnforcer,
}

impl EventHandler {
    pub fn new(
        sdl_waker: Arc<Mutex<Option<EventSender>>>,
        winit_waker: Arc<Mutex<Option<EventLoopProxy<RunnerEvent>>>>,
        gui_dispatcher: Arc<Mutex<Option<GuiDispatcher>>>,
        viiper_address: Option<SocketAddr>,
        async_handle: tokio::runtime::Handle,
        sdl_joystick: sdl3::JoystickSubsystem,
        sdl_gamepad: sdl3::GamepadSubsystem,
    ) -> Self {
        let state = Arc::new(Mutex::new(State {
            devices: HashMap::new(),
            viiper_address,
            binding_enforcer: BindingEnforcer::new(),
        }));
        let bottom_bar = Arc::new(Mutex::new(BottomBar::new()));
        let clone_handle = async_handle.clone();
        let res = Self {
            winit_waker,
            gui_dispatcher,
            async_handle,
            sdl_joystick,
            sdl_gamepad,
            sdl_devices: HashMap::new(),
            sdl_id_to_device: HashMap::new(),
            next_device_id: 1,
            state: state.clone(),
            viiper: ViiperBridge::new(viiper_address, sdl_waker.clone(), clone_handle),
        };
        if let Ok(dispatcher_guard) = res.gui_dispatcher.lock()
            && let Some(dispatcher) = &*dispatcher_guard
        {
            debug!("SDL loop GUI dispatcher initialized");
            dispatcher.register_callback(move |ctx| {
                if let (Ok(mut state_guard), Ok(mut bar_guard)) = (state.lock(), bottom_bar.lock())
                {
                    let state = &mut *state_guard;
                    let bar = &mut *bar_guard;
                    EventHandler::on_draw(state, sdl_waker.clone(), bar, ctx);
                }
            });
        }
        res
    }

    pub(super) fn request_redraw(&self) {
        _ = self
            // the legend of zelda: the
            .winit_waker
            .lock()
            .ok()
            .and_then(|g| g.as_ref().map(|p| p.send_event(RunnerEvent::Redraw())))
            .map(|r| r.inspect_err(|e| warn!("Failed to request GUI redraw: {}", e)));
    }

    pub(super) fn on_draw(
        state: &mut State,
        sdl_waker: Arc<Mutex<Option<EventSender>>>,
        bottom_bar: &mut BottomBar,
        ctx: &egui::Context,
    ) {
        bottom_bar.draw(state, sdl_waker, ctx);
    }
}
