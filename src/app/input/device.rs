use std::fmt::Debug;

use viiper_client::devices::{
    dualshock4::{Dualshock4Input, Dualshock4Output},
    keyboard::{KeyboardInput, KeyboardOutput},
    mouse::MouseInput,
    xbox360::{Xbox360Input, Xbox360Output},
};

use crate::app::input::{device_info::SDLDeviceInfo, device_state::DeviceState};

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
            "dualshock4" => {
                self.state = DeviceState::Dualshock4 {
                    input_state: Dualshock4Input::default(),
                    output_state: Dualshock4Output::default(),
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

// is thread safe, fuck this
unsafe impl Send for SDLDevice {}
unsafe impl Sync for SDLDevice {}

unsafe impl Send for Device {}
unsafe impl Sync for Device {}
