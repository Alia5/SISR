use std::{any::Any, fmt::Debug};

use dashmap::DashMap;
use viiper_client::devices::{
    keyboard::{KeyboardInput, KeyboardOutput},
    mouse::MouseInput,
    steamdeck::{SteamdeckInput, SteamdeckOutput},
    xbox360::{Xbox360Input, Xbox360Output},
};

use crate::app::input::{
    device_update::{self},
    sdl_device_info::SDLDeviceInfo,
};

#[derive(Debug, Default)]
pub struct Device {
    pub id: u64,
    pub sdl_devices: Vec<SDLDevice>,
    pub steam_handle: u64,
    pub viiper_type: Option<String>,
    pub viiper_device: Option<ViiperDevice>,
}

impl Device {
    pub fn sdl_device(&mut self, sdl_id: u32) -> Option<&mut SDLDevice> {
        self.sdl_devices
            .iter_mut()
            .find(|sdl_device| sdl_device.id == sdl_id)
    }
}
pub struct ViiperDevice {
    pub device: viiper_client::Device,
    pub state: DeviceState,
}

impl ViiperDevice {
    pub fn init_state(&mut self) {
        match self.device.r#type.as_str() {
            "xbox360" => {
                self.state = DeviceState::Xbox360 {
                    input_state: Xbox360Input::default(),
                    output_state: Xbox360Output::default(),
                };
            }
            "keyboard" => {
                self.state = DeviceState::Keyboard {
                    input_state: KeyboardInput::default(),
                    output_state: KeyboardOutput::default(),
                };
            }
            "mouse" => {
                self.state = DeviceState::Mouse {
                    input_state: MouseInput::default(),
                };
            }
            "steamdeck" => {
                self.state = DeviceState::SteamDeck {
                    input_state: SteamdeckInput::default(),
                    output_state: SteamdeckOutput::default(),
                };
            }
            _ => {
                tracing::warn!(
                    "Unknown Viiper device type '{}' for device",
                    self.device.r#type,
                );
                self.state = DeviceState::Empty;
            }
        }
    }
}

impl Debug for ViiperDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.device)
    }
}

#[derive(Clone, Default, Debug)]
pub enum DeviceState {
    #[default]
    Empty,
    Xbox360 {
        input_state: Xbox360Input,
        output_state: Xbox360Output,
    },
    Keyboard {
        input_state: KeyboardInput,
        output_state: KeyboardOutput,
    },
    SteamDeck {
        input_state: SteamdeckInput,
        output_state: SteamdeckOutput,
    },
    Mouse {
        input_state: MouseInput,
    },
}

impl DeviceState {
    pub fn is_empty(&self) -> bool {
        matches!(self, DeviceState::Empty)
    }

    pub fn viiper_type(&self) -> Option<&'static str> {
        match self {
            DeviceState::Xbox360 { .. } => Some("xbox360"),
            DeviceState::Keyboard { .. } => Some("keyboard"),
            DeviceState::Mouse { .. } => Some("mouse"),
            DeviceState::SteamDeck { .. } => Some("steamdeck"),
            DeviceState::Empty => None,
        }
    }

    pub fn boxed(&self) -> Option<Box<dyn Any + Send>> {
        match self {
            DeviceState::Xbox360 { input_state, .. } => Some(Box::new(input_state.clone())),
            DeviceState::Keyboard { input_state, .. } => Some(Box::new(input_state.clone())),
            DeviceState::Mouse { input_state } => Some(Box::new(input_state.clone())),
            DeviceState::SteamDeck { input_state, .. } => Some(Box::new(input_state.clone())),
            DeviceState::Empty => None,
        }
    }

    pub fn update_from_sdl_gamepad(&mut self, gp: &sdl3::gamepad::Gamepad) {
        match self {
            DeviceState::Empty => {
                tracing::error!(
                    "Attempted to update Empty controller state from SDL gamepad: {:?}",
                    self
                );
            }
            DeviceState::Xbox360 { input_state, .. } => {
                device_update::xbox360::update_from_sdl_gamepad(input_state, gp);
            }
            DeviceState::SteamDeck { .. } => {
                // ignored
            }
            _ => {
                tracing::warn!(
                    "Attempted not yet implemented controller update: {:?}",
                    self
                );
            }
        }
    }
}

pub struct SDLDevice {
    pub id: u32,
    pub infos: SDLDeviceInfo,
    pub joystick: Option<sdl3::joystick::Joystick>,
    pub gamepad: Option<sdl3::gamepad::Gamepad>,
}

impl SDLDevice {
    pub fn new(
        id: u32,
        joystick: Option<sdl3::joystick::Joystick>,
        gamepad: Option<sdl3::gamepad::Gamepad>,
    ) -> Self {
        let mut res = Self {
            id,
            infos: SDLDeviceInfo::default(),
            joystick,
            gamepad,
        };
        res.update_info();
        res
    }
    pub fn update_info(&mut self) {
        self.infos.update(&self.joystick, &self.gamepad);
    }
}

impl Debug for SDLDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match (&self.joystick, &self.gamepad) {
            (Some(js), None) => f
                .debug_struct("SDLDevice::Joystick")
                .field("name", &js.name())
                .field("id", &js.id())
                .finish(),
            (None, Some(gp)) => f
                .debug_struct("SDLDevice::Gamepad")
                .field("name", &gp.name())
                .field("id", &gp.id())
                .finish(),
            (Some(js), Some(gp)) => f
                .debug_struct("SDLDevice::Joystick+Gamepad")
                .field("id", &self.id)
                .field("joystick_name", &js.name())
                .field("gamepad_name", &gp.name().unwrap_or("N/A".to_string()))
                .finish(),
            _ => write!(f, "SDLDevice::Unknown"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum SdlValue {
    String(String),
    OptString(Option<String>),
    U16(u16),
    OptU16(Option<u16>),
    HexU16(Option<u16>),
    U32(u32),
    Bool(bool),
    Nested(DashMap<String, SdlValue>),
}

impl std::fmt::Display for SdlValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SdlValue::String(s) => write!(f, "{}", s),
            SdlValue::OptString(Some(s)) => write!(f, "{}", s),
            SdlValue::OptString(None) => write!(f, "N/A"),
            SdlValue::U16(v) => write!(f, "{}", v),
            SdlValue::OptU16(Some(v)) => write!(f, "{}", v),
            SdlValue::OptU16(None) => write!(f, "N/A"),
            SdlValue::HexU16(Some(v)) => write!(f, "0x{:04X}", v),
            SdlValue::HexU16(None) => write!(f, "N/A"),
            SdlValue::U32(v) => write!(f, "{}", v),
            SdlValue::Bool(v) => write!(f, "{}", v),
            SdlValue::Nested(map) => write!(f, "({} items)", map.len()),
        }
    }
}

// is thread safe, fuck this
unsafe impl Send for SDLDevice {}
unsafe impl Sync for SDLDevice {}

unsafe impl Send for Device {}
unsafe impl Sync for Device {}
