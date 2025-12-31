use dashmap::DashMap;
use sdl3::{
    gamepad::{Axis, Button},
    joystick,
};

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
