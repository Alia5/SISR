mod events_connection;
mod events_input;
mod events_kbm;
mod events_misc;
mod gui;
mod viiper_bridge;

mod events;
pub use events::HandlerEvent;
use viiper_bridge::ViiperBridge;
pub use viiper_bridge::ViiperEvent;

use std::{
    collections::{BTreeSet, HashMap},
    net::SocketAddr,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use sdl3::event::EventSender;
use tracing::{debug, trace, warn};
use winit::event_loop::EventLoopProxy;

use crate::app::input::handler::gui::bottom_bar::BottomBar;
use crate::app::{
    gui::dispatcher::GuiDispatcher,
    input::device::{Device, DeviceState, SDLDevice},
    window::RunnerEvent,
};
use crate::config::CONFIG;

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
    kbm_emulation_enabled: Arc<AtomicBool>,
    state: Arc<Mutex<State>>,
}

pub(super) struct State {
    devices: HashMap<u64, Device>,
    viiper_address: Option<SocketAddr>,
    cef_debug_port: Option<u16>,
    steam_overlay_open: bool,
    kbm_emulation_enabled: bool,
    kbm_keyboard_modifiers: u8,
    kbm_keyboard_keys: BTreeSet<u8>,
    kbm_mouse_buttons: u8,
    window_continuous_redraw: Arc<AtomicBool>,
    async_handle: tokio::runtime::Handle,
}

impl EventHandler {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        sdl_waker: Arc<Mutex<Option<EventSender>>>,
        winit_waker: Arc<Mutex<Option<EventLoopProxy<RunnerEvent>>>>,
        gui_dispatcher: Arc<Mutex<Option<GuiDispatcher>>>,
        viiper_address: Option<SocketAddr>,
        async_handle: tokio::runtime::Handle,
        sdl_joystick: sdl3::JoystickSubsystem,
        sdl_gamepad: sdl3::GamepadSubsystem,
        window_continuous_redraw: Arc<AtomicBool>,
        kbm_emulation_enabled: Arc<AtomicBool>,
    ) -> Self {
        let state = Arc::new(Mutex::new(State {
            devices: HashMap::new(),
            viiper_address,
            cef_debug_port: None,
            steam_overlay_open: false,
            kbm_emulation_enabled: kbm_emulation_enabled.load(Ordering::Relaxed),
            kbm_keyboard_modifiers: 0,
            kbm_keyboard_keys: BTreeSet::new(),
            kbm_mouse_buttons: 0,
            window_continuous_redraw: window_continuous_redraw.clone(),
            async_handle: async_handle.clone(),
        }));
        let bottom_bar = Arc::new(Mutex::new(BottomBar::new()));
        let clone_handle = async_handle.clone();
        let mut res = Self {
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
            kbm_emulation_enabled: kbm_emulation_enabled.clone(),
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

        if kbm_emulation_enabled.load(Ordering::Relaxed)
            && let Ok(mut guard) = res.state.lock()
        {
            let keyboard_id = res.next_device_id;
            res.next_device_id += 1;
            let mouse_id = res.next_device_id;
            res.next_device_id += 1;

            let keyboard_device = crate::app::input::device::Device {
                id: keyboard_id,
                viiper_type: "keyboard".to_string(),
                ..Default::default()
            };
            let mouse_device = crate::app::input::device::Device {
                id: mouse_id,
                viiper_type: "mouse".to_string(),
                ..Default::default()
            };

            res.viiper.create_device(&keyboard_device);
            res.viiper.create_device(&mouse_device);
            guard.devices.insert(keyboard_id, keyboard_device);
            guard.devices.insert(mouse_id, mouse_device);
        }

        res
    }

    pub(super) fn request_redraw(&self) {
        trace!("Requesting GUI redraw");
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
