use std::fmt::Debug;
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
    pub id: u32,
    pub steam_handle: u64,
    pub state: DeviceState,
    pub sdl_device_count: usize,
    pub viiper_type: String,
    pub viiper_device: Option<viiper_client::types::Device>,
}

impl Default for Device {
    fn default() -> Self {
        Self {
            id: 0,
            steam_handle: 0,
            state: DeviceState::default(),
            sdl_device_count: 0,
            viiper_type: "xbox360".to_string(),
            viiper_device: None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DeviceState {
    pub buttons: u32, // TODO
}
