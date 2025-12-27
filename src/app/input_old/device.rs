use std::{collections::HashSet, fmt::Debug};

use viiper_client::devices::xbox360;

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

#[derive(Debug, Clone, Default)]
pub struct DeviceState {
    pub input: xbox360::Xbox360Input,
}

impl DeviceState {
    pub fn update_from_sdl_gamepad(&mut self, gp: &sdl3::gamepad::Gamepad) {
        let mut b: u32 = 0;

        if gp.button(sdl3::gamepad::Button::South) {
            b |= xbox360::BUTTON_A as u32;
        }
        if gp.button(sdl3::gamepad::Button::East) {
            b |= xbox360::BUTTON_B as u32;
        }
        if gp.button(sdl3::gamepad::Button::West) {
            b |= xbox360::BUTTON_X as u32;
        }
        if gp.button(sdl3::gamepad::Button::North) {
            b |= xbox360::BUTTON_Y as u32;
        }
        if gp.button(sdl3::gamepad::Button::Start) {
            b |= xbox360::BUTTON_START as u32;
        }
        if gp.button(sdl3::gamepad::Button::Back) {
            b |= xbox360::BUTTON_BACK as u32;
        }
        if gp.button(sdl3::gamepad::Button::LeftStick) {
            b |= xbox360::BUTTON_L_THUMB as u32;
        }
        if gp.button(sdl3::gamepad::Button::RightStick) {
            b |= xbox360::BUTTON_R_THUMB as u32;
        }
        if gp.button(sdl3::gamepad::Button::LeftShoulder) {
            b |= xbox360::BUTTON_L_SHOULDER as u32;
        }
        if gp.button(sdl3::gamepad::Button::RightShoulder) {
            b |= xbox360::BUTTON_R_SHOULDER as u32;
        }
        if gp.button(sdl3::gamepad::Button::Guide) {
            b |= xbox360::BUTTON_GUIDE as u32;
        }
        if gp.button(sdl3::gamepad::Button::DPadUp) {
            b |= xbox360::BUTTON_D_PAD_UP as u32;
        }
        if gp.button(sdl3::gamepad::Button::DPadDown) {
            b |= xbox360::BUTTON_D_PAD_DOWN as u32;
        }
        if gp.button(sdl3::gamepad::Button::DPadLeft) {
            b |= xbox360::BUTTON_D_PAD_LEFT as u32;
        }
        if gp.button(sdl3::gamepad::Button::DPadRight) {
            b |= xbox360::BUTTON_D_PAD_RIGHT as u32;
        }

        let lt = gp.axis(sdl3::gamepad::Axis::TriggerLeft);
        let rt = gp.axis(sdl3::gamepad::Axis::TriggerRight);

        self.input.buttons = b;
        self.input.lt = ((lt.max(0) as i32 * 255) / 32767).clamp(0, 255) as u8;
        self.input.rt = ((rt.max(0) as i32 * 255) / 32767).clamp(0, 255) as u8;

        // Invert Y axes to match XInput convention
        // XInput: Negative values signify down or to the left. Positive values signify up or to the right.
        //         https://learn.microsoft.com/en-us/windows/win32/api/xinput/ns-xinput-xinput_gamepad
        // SDL: For thumbsticks, the state is a value ranging from -32768 (up/left) to 32767 (down/right).
        //      https://wiki.libsdl.org/SDL3/SDL_GetGamepadAxis
        self.input.lx = gp.axis(sdl3::gamepad::Axis::LeftX);
        self.input.ly = gp.axis(sdl3::gamepad::Axis::LeftY).saturating_neg();
        self.input.rx = gp.axis(sdl3::gamepad::Axis::RightX);
        self.input.ry = gp.axis(sdl3::gamepad::Axis::RightY).saturating_neg();
    }
}
