use std::{any::Any, fmt::Debug};

use dashmap::DashMap;
use sdl3::{
    gamepad::{Axis, Button},
    joystick,
};
use viiper_client::devices::{
        keyboard::{KeyboardInput, KeyboardOutput},
        mouse::MouseInput,
        xbox360::{self, Xbox360Input, Xbox360Output},
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
            DeviceState::Empty => None,
        }
    }

    pub fn boxed(&self) -> Option<Box<dyn Any + Send>> {
        match self {
            DeviceState::Xbox360 { input_state, .. } => Some(Box::new(input_state.clone())),
            DeviceState::Keyboard { input_state, .. } => Some(Box::new(input_state.clone())),
            DeviceState::Mouse { input_state } => Some(Box::new(input_state.clone())),
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
                Self::update_xbox360_input_state_from_sdl_gamepad(input_state, gp);
            }
            _ => {
                tracing::warn!(
                    "Attempted not yet implemented controller update: {:?}",
                    self
                );
            }
        }
    }

    // TODO: move
    fn update_xbox360_input_state_from_sdl_gamepad(
        istate: &mut Xbox360Input,
        gp: &sdl3::gamepad::Gamepad,
    ) {
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

        istate.buttons = b;
        istate.lt = ((lt.max(0) as i32 * 255) / 32767).clamp(0, 255) as u8;
        istate.rt = ((rt.max(0) as i32 * 255) / 32767).clamp(0, 255) as u8;

        // Invert Y axes to match XInput convention
        // XInput: Negative values signify down or to the left. Positive values signify up or to the right.
        //         https://learn.microsoft.com/en-us/windows/win32/api/xinput/ns-xinput-xinput_gamepad
        // SDL: For thumbsticks, the state is a value ranging from -32768 (up/left) to 32767 (down/right).
        //      https://wiki.libsdl.org/SDL3/SDL_GetGamepadAxis
        istate.lx = gp.axis(sdl3::gamepad::Axis::LeftX);
        istate.ly = gp.axis(sdl3::gamepad::Axis::LeftY).saturating_neg();
        istate.rx = gp.axis(sdl3::gamepad::Axis::RightX);
        istate.ry = gp.axis(sdl3::gamepad::Axis::RightY).saturating_neg();
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

#[derive(Debug, Default)]
pub struct SDLDeviceInfo {
    pub joystick_infos: DashMap<String, SdlValue>,
    pub gamepad_infos: DashMap<String, SdlValue>,
}

impl SDLDeviceInfo {
    pub fn update(
        &mut self,
        joystick: &Option<joystick::Joystick>,
        gamepad: &Option<sdl3::gamepad::Gamepad>,
    ) {
        if let Some(js) = joystick {
            let i = &self.joystick_infos;
            i.insert("name".into(), SdlValue::String(js.name()));
            i.insert("id".into(), SdlValue::U32(js.id()));
            i.insert("guid".into(), SdlValue::String(js.guid().string()));
            i.insert("connected".into(), SdlValue::Bool(js.connected()));
            i.insert("num_axes".into(), SdlValue::U32(js.num_axes()));
            i.insert("num_buttons".into(), SdlValue::U32(js.num_buttons()));
            i.insert("num_hats".into(), SdlValue::U32(js.num_hats()));
            i.insert(
                "has_rumble".into(),
                SdlValue::Bool(unsafe { js.has_rumble() }),
            );
            i.insert(
                "has_rumble_triggers".into(),
                SdlValue::Bool(unsafe { js.has_rumble_triggers() }),
            );
            if let Ok(power) = js.power_info() {
                i.insert(
                    "power_info".into(),
                    SdlValue::String(format!("{:?}", power)),
                );
            }

            let axes = DashMap::new();
            for i in 0..js.num_axes() {
                axes.insert(format!("Axis {}", i), SdlValue::String("✅".into()));
            }
            i.insert("axes".into(), SdlValue::Nested(axes));

            let buttons = DashMap::new();
            for i in 0..js.num_buttons() {
                buttons.insert(format!("Button {}", i), SdlValue::String("✅".into()));
            }
            i.insert("buttons".into(), SdlValue::Nested(buttons));

            let hats = DashMap::new();
            for i in 0..js.num_hats() {
                hats.insert(format!("Hat {}", i), SdlValue::String("✅".into()));
            }
            i.insert("hats".into(), SdlValue::Nested(hats));
        }

        if let Some(gp) = gamepad {
            let i = &self.gamepad_infos;
            i.insert("name".into(), SdlValue::OptString(gp.name()));
            i.insert("id".into(), SdlValue::U32(gp.id().unwrap_or(0)));
            i.insert("path".into(), SdlValue::OptString(gp.path()));
            i.insert("type".into(), SdlValue::String(gp.r#type().string()));
            i.insert(
                "real_type".into(),
                SdlValue::String(gp.real_type().string()),
            );
            i.insert("connected".into(), SdlValue::Bool(gp.connected()));
            i.insert("vendor_id".into(), SdlValue::HexU16(gp.vendor_id()));
            i.insert("product_id".into(), SdlValue::HexU16(gp.product_id()));
            i.insert(
                "product_version".into(),
                SdlValue::OptU16(gp.product_version()),
            );
            i.insert(
                "firmware_version".into(),
                SdlValue::OptU16(gp.firmware_version()),
            );
            i.insert(
                "serial_number".into(),
                SdlValue::OptString(gp.serial_number()),
            );
            i.insert("player_index".into(), SdlValue::OptU16(gp.player_index()));
            i.insert(
                "has_rumble".into(),
                SdlValue::Bool(unsafe { gp.has_rumble() }),
            );
            i.insert(
                "has_rumble_triggers".into(),
                SdlValue::Bool(unsafe { gp.has_rumble_triggers() }),
            );
            let touchpads = gp.touchpads_count();
            i.insert("has_touchpads".into(), SdlValue::Bool(touchpads > 0));
            i.insert("touchpads_count".into(), SdlValue::U16(touchpads));
            let power = gp.power_info();
            i.insert(
                "power_info".into(),
                SdlValue::String(format!("{:?}", power)),
            );
            if let Some(mapping) = gp.mapping() {
                i.insert("mapping".into(), SdlValue::String(mapping));
            }

            let axes = DashMap::new();
            for axis in [
                Axis::LeftX,
                Axis::LeftY,
                Axis::RightX,
                Axis::RightY,
                Axis::TriggerLeft,
                Axis::TriggerRight,
            ] {
                if gp.has_axis(axis) {
                    let name = axis.string();
                    axes.insert(name.clone(), SdlValue::String(name));
                }
            }
            i.insert("axes".into(), SdlValue::Nested(axes));

            let buttons = DashMap::new();
            for button in [
                Button::South,
                Button::East,
                Button::West,
                Button::North,
                Button::Back,
                Button::Guide,
                Button::Start,
                Button::LeftStick,
                Button::RightStick,
                Button::LeftShoulder,
                Button::RightShoulder,
                Button::DPadUp,
                Button::DPadDown,
                Button::DPadLeft,
                Button::DPadRight,
                Button::Misc1,
                Button::Misc2,
                Button::Misc3,
                Button::Misc4,
                Button::Misc5,
                Button::RightPaddle1,
                Button::LeftPaddle1,
                Button::RightPaddle2,
                Button::LeftPaddle2,
                Button::Touchpad,
            ] {
                if gp.has_button(button) {
                    let name = button.string();
                    buttons.insert(name.clone(), SdlValue::String(name));
                }
            }
            i.insert("buttons".into(), SdlValue::Nested(buttons));
        }
    }
}

// is thread safe, fuck this
unsafe impl Send for SDLDevice {}
unsafe impl Sync for SDLDevice {}

unsafe impl Send for Device {}
unsafe impl Sync for Device {}
