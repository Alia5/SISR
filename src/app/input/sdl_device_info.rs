use dashmap::DashMap;
use sdl3::gamepad::{Axis, Button};
use sdl3::sys::{gamepad as sys_gamepad, sensor as sys_sensor};

use crate::app::input::device::SdlValue;

#[derive(Debug, Default)]
pub struct SDLDeviceInfo {
    pub joystick_infos: DashMap<String, SdlValue>,
    pub gamepad_infos: DashMap<String, SdlValue>,
}

impl SDLDeviceInfo {
    pub fn update(
        &mut self,
        joystick: &Option<sdl3::joystick::Joystick>,
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

            if let Some(raw) = unsafe { raw_sdl_gamepad_ptr(gp) } {
                insert_gamepad_touchpad_capabilities(i, raw);
                insert_gamepad_sensor_capabilities(i, raw);
                insert_gamepad_face_button_labels(i, raw);
            }

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

unsafe fn raw_sdl_gamepad_ptr(
    pad: &sdl3::gamepad::Gamepad,
) -> Option<*mut sdl3::sys::gamepad::SDL_Gamepad> {
    #[repr(C)]
    struct GamepadRaw {
        _subsystem: [u8; std::mem::size_of::<sdl3::GamepadSubsystem>()],
        raw: *mut sdl3::sys::gamepad::SDL_Gamepad,
    }

    let gamepad_raw: &GamepadRaw = unsafe { std::mem::transmute(pad) };
    if gamepad_raw.raw.is_null() {
        return None;
    }
    Some(gamepad_raw.raw)
}

fn insert_gamepad_touchpad_capabilities(
    infos: &DashMap<String, SdlValue>,
    raw: *mut sdl3::sys::gamepad::SDL_Gamepad,
) {
    let touchpads = unsafe { sys_gamepad::SDL_GetNumGamepadTouchpads(raw) };
    if touchpads <= 0 {
        return;
    }

    let touchpads_map = DashMap::new();
    for touchpad in 0..touchpads {
        let fingers = unsafe { sys_gamepad::SDL_GetNumGamepadTouchpadFingers(raw, touchpad) };
        let tp = DashMap::new();
        tp.insert("fingers_max".into(), SdlValue::U32(fingers.max(0) as u32));
        touchpads_map.insert(format!("Touchpad {}", touchpad), SdlValue::Nested(tp));
    }

    infos.insert("touchpads".into(), SdlValue::Nested(touchpads_map));
}

fn insert_gamepad_sensor_capabilities(
    infos: &DashMap<String, SdlValue>,
    raw: *mut sdl3::sys::gamepad::SDL_Gamepad,
) {
    let sensors = [
        ("accel", sys_sensor::SDL_SensorType::ACCEL),
        ("gyro", sys_sensor::SDL_SensorType::GYRO),
        ("accel_l", sys_sensor::SDL_SensorType::ACCEL_L),
        ("gyro_l", sys_sensor::SDL_SensorType::GYRO_L),
        ("accel_r", sys_sensor::SDL_SensorType::ACCEL_R),
        ("gyro_r", sys_sensor::SDL_SensorType::GYRO_R),
    ];

    let sensors_map = DashMap::new();
    let mut any = false;

    for (label, sensor_type) in sensors {
        let has = unsafe { sys_gamepad::SDL_GamepadHasSensor(raw, sensor_type) };
        if !has {
            continue;
        }
        any = true;

        let enabled = unsafe { sys_gamepad::SDL_GamepadSensorEnabled(raw, sensor_type) };
        let rate = unsafe { sys_gamepad::SDL_GetGamepadSensorDataRate(raw, sensor_type) };

        let entry = DashMap::new();
        entry.insert("has".into(), SdlValue::Bool(true));
        entry.insert("enabled".into(), SdlValue::Bool(enabled));
        entry.insert(
            "data_rate_hz".into(),
            SdlValue::String(format!("{:.2}", rate)),
        );
        sensors_map.insert(label.to_string(), SdlValue::Nested(entry));
    }

    infos.insert("has_sensors".into(), SdlValue::Bool(any));
    if any {
        infos.insert("sensors".into(), SdlValue::Nested(sensors_map));
    }
}

fn gamepad_button_label_to_string(label: sys_gamepad::SDL_GamepadButtonLabel) -> &'static str {
    match label {
        sys_gamepad::SDL_GamepadButtonLabel::UNKNOWN => "unknown",
        sys_gamepad::SDL_GamepadButtonLabel::A => "A",
        sys_gamepad::SDL_GamepadButtonLabel::B => "B",
        sys_gamepad::SDL_GamepadButtonLabel::X => "X",
        sys_gamepad::SDL_GamepadButtonLabel::Y => "Y",
        sys_gamepad::SDL_GamepadButtonLabel::CROSS => "Cross",
        sys_gamepad::SDL_GamepadButtonLabel::CIRCLE => "Circle",
        sys_gamepad::SDL_GamepadButtonLabel::SQUARE => "Square",
        sys_gamepad::SDL_GamepadButtonLabel::TRIANGLE => "Triangle",
        _ => "unknown",
    }
}

fn insert_gamepad_face_button_labels(
    infos: &DashMap<String, SdlValue>,
    raw: *mut sdl3::sys::gamepad::SDL_Gamepad,
) {
    let labels = [
        ("south", sys_gamepad::SDL_GamepadButton::SOUTH),
        ("east", sys_gamepad::SDL_GamepadButton::EAST),
        ("west", sys_gamepad::SDL_GamepadButton::WEST),
        ("north", sys_gamepad::SDL_GamepadButton::NORTH),
    ];

    let labels_map = DashMap::new();
    for (pos, btn) in labels {
        let label = unsafe { sys_gamepad::SDL_GetGamepadButtonLabel(raw, btn) };
        labels_map.insert(
            pos.to_string(),
            SdlValue::String(gamepad_button_label_to_string(label).to_string()),
        );
    }

    infos.insert("face_button_labels".into(), SdlValue::Nested(labels_map));
}
