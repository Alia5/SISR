use sdl3_sys::joystick::SDL_GetJoystickFromID;
use viiper_client::devices::steamdeck::SteamdeckInput;

use crate::app::input::sdl_sisr::ValveInReport;

pub fn update_from_raw_event(istate: &mut SteamdeckInput, raw_data: &[u8; 64]) -> bool {
    if raw_data.len() < std::mem::size_of::<ValveInReport>() {
        tracing::error!("Raw event data too small for ValveInReport");
        return false;
    }

    let report = unsafe { std::ptr::read_unaligned(raw_data.as_ptr() as *const ValveInReport) };

    let (packet_num, deck_state) = unsafe {
        let packet_num = report.payload.deck_state.packet_num;
        let deck_state = &report.payload.deck_state;
        (packet_num, deck_state)
    };

    tracing::trace!("Updating raw steamdeck state, seqno {}", packet_num);

    istate.left_pad_x = deck_state.left_pad_x;
    istate.left_pad_y = deck_state.left_pad_y;
    istate.right_pad_x = deck_state.right_pad_x;
    istate.right_pad_y = deck_state.right_pad_y;
    istate.pressure_pad_left = deck_state.pressure_pad_left;
    istate.pressure_pad_right = deck_state.pressure_pad_right;

    istate.accel_x = deck_state.accel_x;
    istate.accel_y = deck_state.accel_y;
    istate.accel_z = deck_state.accel_z;

    istate.gyro_x = deck_state.gyro_x;
    istate.gyro_y = deck_state.gyro_y;
    istate.gyro_z = deck_state.gyro_z;

    istate.gyro_quat_w = deck_state.gyro_quat_w;
    istate.gyro_quat_x = deck_state.gyro_quat_x;
    istate.gyro_quat_y = deck_state.gyro_quat_y;
    istate.gyro_quat_z = deck_state.gyro_quat_z;

    istate.buttons = ((deck_state.buttons_h as u64) << 32) | (deck_state.buttons_l as u64);

    istate.left_stick_x = deck_state.left_stick_x;
    istate.left_stick_y = deck_state.left_stick_y;
    istate.right_stick_x = deck_state.right_stick_x;
    istate.right_stick_y = deck_state.right_stick_y;
    istate.trigger_l = deck_state.trigger_raw_l;
    istate.trigger_r = deck_state.trigger_raw_r;

    true
}
