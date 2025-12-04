mod events_connection;
mod events_input;
mod gui;
mod viiper_bridge;

use viiper_bridge::ViiperBridge;
pub use viiper_bridge::ViiperEvent;

use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use sdl3::event::EventSender;
use tracing::debug;
use winit::event_loop::EventLoopProxy;

use crate::app::{
    gui::dispatcher::GuiDispatcher,
    input::device::{Device, SDLDevice},
    steam_utils::BindingEnforcer,
    window::RunnerEvent,
};

pub struct EventHandler {
    winit_waker: Arc<Mutex<Option<EventLoopProxy<RunnerEvent>>>>,
    gui_dispatcher: Arc<Mutex<Option<GuiDispatcher>>>,
    async_handle: tokio::runtime::Handle,
    sdl_joystick: sdl3::JoystickSubsystem,
    sdl_gamepad: sdl3::GamepadSubsystem,
    sdl_devices: HashMap<u32, Vec<SDLDevice>>,
    sdl_id_to_device: HashMap<u32, u64>,
    next_device_id: u64,
    viiper: ViiperBridge,
    state: Arc<Mutex<State>>,
}

pub(super) struct State {
    devices: Vec<Device>,
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
            devices: Vec::new(),
            viiper_address,
            binding_enforcer: BindingEnforcer::new(),
        }));
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
            viiper: ViiperBridge::new(viiper_address, sdl_waker, clone_handle),
        };
        if let Ok(dispatcher_guard) = res.gui_dispatcher.lock()
            && let Some(dispatcher) = &*dispatcher_guard
        {
            debug!("SDL loop GUI dispatcher initialized");
            dispatcher.register_callback(move |ctx| {
                if let Ok(mut guard) = state.lock() {
                    let state = &mut *guard;
                    EventHandler::on_draw(state, ctx);
                }
            });
        }
        res
    }
}
