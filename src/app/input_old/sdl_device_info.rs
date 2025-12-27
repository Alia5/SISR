use std::collections::HashMap;

use sdl3::gamepad::{Axis, Button};

use super::device::SDLDevice;

#[derive(Debug, Clone)]
pub enum SdlValue {
    String(String),
    OptString(Option<String>),
    U16(u16),
    OptU16(Option<u16>),
    HexU16(Option<u16>),
    U32(u32),
    Bool(bool),
    Nested(HashMap<String, SdlValue>),
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

#[derive(Debug, Clone)]
pub struct SdlDeviceInfo {
    pub is_gamepad: bool,
    pub properties: HashMap<String, SdlValue>,
}

impl From<&SDLDevice> for SdlDeviceInfo {
    fn from(device: &SDLDevice) -> Self {
        let mut properties = HashMap::new();

        match device {
            SDLDevice::Joystick(js) => {
                properties.insert("name".into(), SdlValue::String(js.name()));
                properties.insert("id".into(), SdlValue::U32(js.id()));
                properties.insert("guid".into(), SdlValue::String(js.guid().string()));
                properties.insert("connected".into(), SdlValue::Bool(js.connected()));
                properties.insert("num_axes".into(), SdlValue::U32(js.num_axes()));
                properties.insert("num_buttons".into(), SdlValue::U32(js.num_buttons()));
                properties.insert("num_hats".into(), SdlValue::U32(js.num_hats()));
                properties.insert(
                    "has_rumble".into(),
                    SdlValue::Bool(unsafe { js.has_rumble() }),
                );
                properties.insert(
                    "has_rumble_triggers".into(),
                    SdlValue::Bool(unsafe { js.has_rumble_triggers() }),
                );
                if let Ok(power) = js.power_info() {
                    properties.insert(
                        "power_info".into(),
                        SdlValue::String(format!("{:?}", power)),
                    );
                }

                let mut axes = HashMap::new();
                for i in 0..js.num_axes() {
                    axes.insert(format!("Axis {}", i), SdlValue::String("✅".into()));
                }
                properties.insert("axes".into(), SdlValue::Nested(axes));

                let mut buttons = HashMap::new();
                for i in 0..js.num_buttons() {
                    buttons.insert(format!("Button {}", i), SdlValue::String("✅".into()));
                }
                properties.insert("buttons".into(), SdlValue::Nested(buttons));

                let mut hats = HashMap::new();
                for i in 0..js.num_hats() {
                    hats.insert(format!("Hat {}", i), SdlValue::String("✅".into()));
                }
                properties.insert("hats".into(), SdlValue::Nested(hats));

                SdlDeviceInfo {
                    is_gamepad: false,
                    properties,
                }
            }
            SDLDevice::Gamepad(gp) => {
                properties.insert("name".into(), SdlValue::OptString(gp.name()));
                properties.insert("id".into(), SdlValue::U32(gp.id().unwrap_or(0)));
                properties.insert("path".into(), SdlValue::OptString(gp.path()));
                properties.insert("type".into(), SdlValue::String(gp.r#type().string()));
                properties.insert(
                    "real_type".into(),
                    SdlValue::String(gp.real_type().string()),
                );
                properties.insert("connected".into(), SdlValue::Bool(gp.connected()));
                properties.insert("vendor_id".into(), SdlValue::HexU16(gp.vendor_id()));
                properties.insert("product_id".into(), SdlValue::HexU16(gp.product_id()));
                properties.insert(
                    "product_version".into(),
                    SdlValue::OptU16(gp.product_version()),
                );
                properties.insert(
                    "firmware_version".into(),
                    SdlValue::OptU16(gp.firmware_version()),
                );
                properties.insert(
                    "serial_number".into(),
                    SdlValue::OptString(gp.serial_number()),
                );
                properties.insert("player_index".into(), SdlValue::OptU16(gp.player_index()));
                properties.insert(
                    "has_rumble".into(),
                    SdlValue::Bool(unsafe { gp.has_rumble() }),
                );
                properties.insert(
                    "has_rumble_triggers".into(),
                    SdlValue::Bool(unsafe { gp.has_rumble_triggers() }),
                );
                let touchpads = gp.touchpads_count();
                properties.insert("has_touchpads".into(), SdlValue::Bool(touchpads > 0));
                properties.insert("touchpads_count".into(), SdlValue::U16(touchpads));
                let power = gp.power_info();
                properties.insert(
                    "power_info".into(),
                    SdlValue::String(format!("{:?}", power)),
                );
                if let Some(mapping) = gp.mapping() {
                    properties.insert("mapping".into(), SdlValue::String(mapping));
                }

                let mut axes = HashMap::new();
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
                properties.insert("axes".into(), SdlValue::Nested(axes));

                let mut buttons = HashMap::new();
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
                properties.insert("buttons".into(), SdlValue::Nested(buttons));

                SdlDeviceInfo {
                    is_gamepad: true,
                    properties,
                }
            }
        }
    }
}
