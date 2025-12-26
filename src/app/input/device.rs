use std::{collections::HashSet, fmt::Debug};

use tracing::{debug, warn};
use viiper_client::devices::{steamdeck, xbox360};

use crate::app::input::state_mapper;

use super::sdl_device_info::SdlDeviceInfo;

pub enum SDLDevice {
    Joystick(sdl3::joystick::Joystick),
    Gamepad(sdl3::gamepad::Gamepad),
}

impl Debug for SDLDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SDLDevice::Joystick(joystick) => f
                .debug_struct("SDLDevice::Joystick")
                .field("name", &joystick.name())
                .field("id", &joystick.id())
                .finish(),
            SDLDevice::Gamepad(gamepad) => f
                .debug_struct("SDLDevice::Gamepad")
                .field("name", &gamepad.name())
                .field("id", &gamepad.id())
                .finish(),
        }
    }
}

#[derive(Debug)]
pub struct Device {
    pub id: u64,               // internal device_id
    pub sdl_ids: HashSet<u32>, // set of SDL instance IDs (event.which) associated with this device
    pub steam_handle: u64,
    pub viiper_type: String,
    pub viiper_device: Option<viiper_client::types::Device>,
    pub viiper_connected: bool,
    pub sdl_device_infos: Vec<SdlDeviceInfo>,
}

impl Default for Device {
    fn default() -> Self {
        Self {
            id: 0,
            sdl_ids: HashSet::new(),
            steam_handle: 0,
            viiper_type: "xbox360".to_string(),
            viiper_device: None,
            viiper_connected: false,
            sdl_device_infos: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum InputState {
    Xbox360Input(xbox360::Xbox360Input),
    SteamDeckInput(steamdeck::SteamdeckInput),
}

#[derive(Debug, Clone)]
pub struct DeviceState {
    pub input: InputState,
}

impl DeviceState {
    pub fn default() -> Self {
        Self {
            input: InputState::Xbox360Input(xbox360::Xbox360Input::default()),
        }
    }

    pub fn update_from_sdl_gamepad(&mut self, gp: &sdl3::gamepad::Gamepad) {
        match &mut self.input {
            InputState::Xbox360Input(input) => {
                state_mapper::xbox360::update_from_sdl_gamepad(input, gp);
            }
            InputState::SteamDeckInput(input) => {
                state_mapper::steamdeck::update_from_sdl_gamepad(input, gp);
            }
        }
    }
}
