// SIstructs/consts for SISR pecific extensions to SDL

use sdl3_sys::events::SDL_EventType;

pub const SDL_EVENT_JOYSTICK_RAW_INPUT: u32 = 0x7FF0;

#[repr(C)]
pub struct SDL_JoyRawEvent {
    pub type_: SDL_EventType,
    pub reserved: u32,
    pub timestamp: u64,
    pub which: u32,
    pub data: [u8; 64],
}
#[repr(C)]
pub struct ValveInReportHeader {
    pub report_version: u16,
    pub report_type: u8,
    pub length: u8,
}

#[repr(C)]
pub struct SteamDeckStatePacket {
    pub packet_num: u32,
    pub buttons_l: u32,
    pub buttons_h: u32,
    pub left_pad_x: i16,
    pub left_pad_y: i16,
    pub right_pad_x: i16,
    pub right_pad_y: i16,
    pub accel_x: i16,
    pub accel_y: i16,
    pub accel_z: i16,
    pub gyro_x: i16,
    pub gyro_y: i16,
    pub gyro_z: i16,
    pub gyro_quat_w: i16,
    pub gyro_quat_x: i16,
    pub gyro_quat_y: i16,
    pub gyro_quat_z: i16,
    pub trigger_raw_l: u16,
    pub trigger_raw_r: u16,
    pub left_stick_x: i16,
    pub left_stick_y: i16,
    pub right_stick_x: i16,
    pub right_stick_y: i16,
    pub pressure_pad_left: u16,
    pub pressure_pad_right: u16,
}

#[repr(C)]
pub union ValveInReportPayload {
    pub deck_state: std::mem::ManuallyDrop<SteamDeckStatePacket>,
}

#[repr(C)]
pub struct ValveInReport {
    pub header: ValveInReportHeader,
    pub payload: ValveInReportPayload,
}
