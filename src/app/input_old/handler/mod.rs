mod events_connection;
mod events_input;
mod events_kbm;
mod events_misc;
mod gui;
mod viiper_bridge;

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

use crate::app::{
    gui::dispatcher::GuiDispatcher,
    input_old::device::{Device, DeviceState, SDLDevice},
    window::RunnerEvent,
};
use crate::app::{input_old::handler::gui::bottom_bar::BottomBar, window};

pub struct EventHandler {
    gui_dispatcher: Arc<Mutex<Option<GuiDispatcher>>>,
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
    viiper_ready: bool,
    viiper_version: Option<String>,
}

impl EventHandler {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        gui_dispatcher: Arc<Mutex<Option<GuiDispatcher>>>,
        viiper_address: Option<SocketAddr>,
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
            viiper_ready: false,
            viiper_version: None,
        }));
        let bottom_bar = Arc::new(Mutex::new(BottomBar::new()));
        let mut res = Self {
            gui_dispatcher,
            sdl_joystick,
            sdl_gamepad,
            sdl_devices: HashMap::new(),
            sdl_id_to_device: HashMap::new(),
            next_device_id: 1,
            state: state.clone(),
            viiper: ViiperBridge::new(viiper_address),
            kbm_emulation_enabled: kbm_emulation_enabled.clone(),
        };

        if let Ok(dispatcher_guard) = res.gui_dispatcher.lock()
            && let Some(dispatcher) = &*dispatcher_guard
        {
            tracing::debug!("SDL loop GUI dispatcher initialized");
            dispatcher.register_callback(move |ctx| {
                if let (Ok(mut state_guard), Ok(mut bar_guard)) = (state.lock(), bottom_bar.lock())
                {
                    let state = &mut *state_guard;
                    let bar = &mut *bar_guard;
                    EventHandler::on_draw(state, bar, ctx);
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

            let keyboard_device = crate::app::input_old::device::Device {
                id: keyboard_id,
                viiper_type: "keyboard".to_string(),
                ..Default::default()
            };
            let mouse_device = crate::app::input_old::device::Device {
                id: mouse_id,
                viiper_type: "mouse".to_string(),
                ..Default::default()
            };

            if guard.viiper_ready {
                res.viiper.create_device(&keyboard_device);
                res.viiper.create_device(&mouse_device);
            } else {
                tracing::trace!(
                    "VIIPER not ready; scheduling KB/M devices ({} and {}) for connect on ready",
                    keyboard_id,
                    mouse_id
                );
            }
            guard.devices.insert(keyboard_id, keyboard_device);
            guard.devices.insert(mouse_id, mouse_device);
        }

        res
    }

    pub(super) fn request_redraw(&self) {
        tracing::trace!("Requesting GUI redraw");

        if let Err(e) = window::get_event_sender().send_event(RunnerEvent::Redraw()) {
            tracing::warn!("Failed to request GUI redraw: {}", e);
        }
    }

    pub(super) fn on_draw(state: &mut State, bottom_bar: &mut BottomBar, ctx: &egui::Context) {
        bottom_bar.draw(state, ctx);
    }
}
