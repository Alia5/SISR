use sdl3::gamepad::{Axis, Button};
use viiper_client::devices::steamdeck;

pub fn update_from_sdl_gamepad(input: &mut steamdeck::SteamdeckInput, gp: &sdl3::gamepad::Gamepad) {
    // Steam Deck controller reports use a bitmask + a bunch of analog fields.
    // SDL's Gamepad API does not expose Deck trackpads/gyro/accel in a way we currently store
    // in `DeviceState`, so those values are left as 0 for now.
    //
    // Buttons/axes mapping is based on SDL's generic gamepad layout.

    #[inline]
    fn trigger_axis_to_u16(v: i16) -> u16 {
        // SDL3 axis range: [-32768, 32767]. Triggers are typically [0, 32767] but clamp anyway.
        let v = (v.max(0) as i32).clamp(0, 32767);
        ((v * 65535) / 32767).clamp(0, 65535) as u16
    }

    let mut b: u64 = 0;

    if gp.button(Button::South) {
        b |= steamdeck::BUTTON_A;
    }
    if gp.button(Button::East) {
        b |= steamdeck::BUTTON_B;
    }
    if gp.button(Button::West) {
        b |= steamdeck::BUTTON_X;
    }
    if gp.button(Button::North) {
        b |= steamdeck::BUTTON_Y;
    }
    if gp.button(Button::Back) {
        b |= steamdeck::BUTTON_VIEW;
    }
    if gp.button(Button::Start) {
        b |= steamdeck::BUTTON_MENU;
    }
    if gp.button(Button::Guide) {
        b |= steamdeck::BUTTON_STEAM;
    }
    if gp.button(Button::DPadUp) {
        b |= steamdeck::BUTTON_D_PAD_UP;
    }
    if gp.button(Button::DPadRight) {
        b |= steamdeck::BUTTON_D_PAD_RIGHT;
    }
    if gp.button(Button::DPadLeft) {
        b |= steamdeck::BUTTON_D_PAD_LEFT;
    }
    if gp.button(Button::DPadDown) {
        b |= steamdeck::BUTTON_D_PAD_DOWN;
    }
    if gp.button(Button::LeftShoulder) {
        b |= steamdeck::BUTTON_LB;
    }
    if gp.button(Button::RightShoulder) {
        b |= steamdeck::BUTTON_RB;
    }
    if gp.button(Button::LeftStick) {
        b |= steamdeck::BUTTON_L3;
    }
    if gp.button(Button::RightStick) {
        b |= steamdeck::BUTTON_R3;
    }
    if gp.button(Button::LeftPaddle1) {
        b |= steamdeck::BUTTON_L4;
    }
    if gp.button(Button::LeftPaddle2) {
        b |= steamdeck::BUTTON_L5;
    }
    if gp.button(Button::RightPaddle1) {
        b |= steamdeck::BUTTON_R4;
    }
    if gp.button(Button::RightPaddle2) {
        b |= steamdeck::BUTTON_R5;
    }
    if gp.button(Button::Touchpad) {
        b |= steamdeck::BUTTON_RIGHT_PAD_CLICK;
    }
    if gp.button(Button::Misc1) {
        b |= steamdeck::BUTTON_QAM;
    }

    let lt_raw = gp.axis(Axis::TriggerLeft);
    let rt_raw = gp.axis(Axis::TriggerRight);
    let lt_u16 = trigger_axis_to_u16(lt_raw);
    let rt_u16 = trigger_axis_to_u16(rt_raw);

    // TODO:!
    const TRIGGER_BUTTON_THRESHOLD: i16 = 4096;
    if lt_raw > TRIGGER_BUTTON_THRESHOLD {
        b |= steamdeck::BUTTON_L2;
    }
    if rt_raw > TRIGGER_BUTTON_THRESHOLD {
        b |= steamdeck::BUTTON_R2;
    }

    input.buttons = b;

    input.left_stick_x = gp.axis(Axis::LeftX);
    input.left_stick_y = gp.axis(Axis::LeftY);
    input.right_stick_x = gp.axis(Axis::RightX);
    input.right_stick_y = gp.axis(Axis::RightY);

    input.trigger_l = lt_u16;
    input.trigger_r = rt_u16;

    // TODO: Not available via current SDL polling path (requires sensor/touchpad events state):
    input.left_pad_x = 0;
    input.left_pad_y = 0;
    input.right_pad_x = 0;
    input.right_pad_y = 0;
    input.pressure_pad_left = 0;
    input.pressure_pad_right = 0;

    input.accel_x = 0;
    input.accel_y = 0;
    input.accel_z = 0;
    input.gyro_x = 0;
    input.gyro_y = 0;
    input.gyro_z = 0;
    input.gyro_quat_w = 0;
    input.gyro_quat_x = 0;
    input.gyro_quat_y = 0;
    input.gyro_quat_z = 0;
}
