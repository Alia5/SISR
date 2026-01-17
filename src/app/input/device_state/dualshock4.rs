use viiper_client::devices::dualshock4::{self, Dualshock4Input};

pub fn update_from_sdl_gamepad(istate: &mut Dualshock4Input, gp: &sdl3::gamepad::Gamepad) {
    istate.buttons = 0;
    if gp.button(sdl3::gamepad::Button::South) {
        istate.buttons |= dualshock4::BUTTON_CROSS;
    }
    if gp.button(sdl3::gamepad::Button::East) {
        istate.buttons |= dualshock4::BUTTON_CIRCLE;
    }
    if gp.button(sdl3::gamepad::Button::West) {
        istate.buttons |= dualshock4::BUTTON_SQUARE;
    }
    if gp.button(sdl3::gamepad::Button::North) {
        istate.buttons |= dualshock4::BUTTON_TRIANGLE;
    }
    if gp.button(sdl3::gamepad::Button::Start) {
        istate.buttons |= dualshock4::BUTTON_OPTIONS;
    }
    if gp.button(sdl3::gamepad::Button::Back) {
        istate.buttons |= dualshock4::BUTTON_SHARE;
    }
    if gp.button(sdl3::gamepad::Button::LeftStick) {
        istate.buttons |= dualshock4::BUTTON_L3;
    }
    if gp.button(sdl3::gamepad::Button::RightStick) {
        istate.buttons |= dualshock4::BUTTON_R3;
    }
    if gp.button(sdl3::gamepad::Button::LeftShoulder) {
        istate.buttons |= dualshock4::BUTTON_L1;
    }
    if gp.button(sdl3::gamepad::Button::RightShoulder) {
        istate.buttons |= dualshock4::BUTTON_R1;
    }
    if gp.button(sdl3::gamepad::Button::Guide) {
        istate.buttons |= dualshock4::BUTTON_PS;
    }

    istate.dpad = 0;
    if gp.button(sdl3::gamepad::Button::DPadUp) {
        istate.dpad |= dualshock4::D_PAD_UP;
    }
    if gp.button(sdl3::gamepad::Button::DPadDown) {
        istate.dpad |= dualshock4::D_PAD_DOWN;
    }
    if gp.button(sdl3::gamepad::Button::DPadLeft) {
        istate.dpad |= dualshock4::D_PAD_LEFT;
    }
    if gp.button(sdl3::gamepad::Button::DPadRight) {
        istate.dpad |= dualshock4::D_PAD_RIGHT;
    }

    istate.trigger_l2 = ((gp.axis(sdl3::gamepad::Axis::TriggerLeft).max(0) as i32 * 255) / 32767)
        .clamp(0, 255) as u8;
    istate.trigger_r2 = ((gp.axis(sdl3::gamepad::Axis::TriggerRight).max(0) as i32 * 255) / 32767)
        .clamp(0, 255) as u8;

    istate.stick_lx =
        ((gp.axis(sdl3::gamepad::Axis::LeftX) as i32 * 128) / 32767).clamp(-128, 127) as i8;
    istate.stick_ly =
        (((gp.axis(sdl3::gamepad::Axis::LeftY) as i32) * 128) / 32767).clamp(-128, 127) as i8;
    istate.stick_rx =
        ((gp.axis(sdl3::gamepad::Axis::RightX) as i32 * 128) / 32767).clamp(-128, 127) as i8;
    istate.stick_ry =
        (((gp.axis(sdl3::gamepad::Axis::RightY) as i32) * 128) / 32767).clamp(-128, 127) as i8;

    // TODO: touchpad, gyro, accel
}
