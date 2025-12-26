use viiper_client::devices::xbox360;

pub fn update_from_sdl_gamepad(input: &mut xbox360::Xbox360Input, gp: &sdl3::gamepad::Gamepad) {
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

    input.buttons = b;
    input.lt = ((lt.max(0) as i32 * 255) / 32767).clamp(0, 255) as u8;
    input.rt = ((rt.max(0) as i32 * 255) / 32767).clamp(0, 255) as u8;

    // Invert Y axes to match XInput convention
    // XInput: Negative values signify down or to the left. Positive values signify up or to the right.
    //         https://learn.microsoft.com/en-us/windows/win32/api/xinput/ns-xinput-xinput_gamepad
    // SDL: For thumbsticks, the state is a value ranging from -32768 (up/left) to 32767 (down/right).
    //      https://wiki.libsdl.org/SDL3/SDL_GetGamepadAxis
    input.lx = gp.axis(sdl3::gamepad::Axis::LeftX);
    input.ly = gp.axis(sdl3::gamepad::Axis::LeftY).saturating_neg();
    input.rx = gp.axis(sdl3::gamepad::Axis::RightX);
    input.ry = gp.axis(sdl3::gamepad::Axis::RightY).saturating_neg();
}
