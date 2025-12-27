use sdl3::gamepad::Gamepad;

pub fn get_gamepad_steam_handle(pad: &Gamepad) -> u64 {
    use sdl3::sys::gamepad::SDL_GetGamepadSteamHandle;
    let instance_id = pad.id().unwrap_or(0);
    if instance_id == 0 {
        tracing::trace!("Cannot get steam handle for device with invalid instance ID 0");
        return 0;
    }

    unsafe {
        // Extract the raw SDL_Gamepad pointer from the opened gamepad
        // sdl3-0.16.2\src\sdl3\gamepad.rs:745
        #[repr(C)]
        struct GamepadRaw {
            _subsystem: [u8; std::mem::size_of::<sdl3::GamepadSubsystem>()],
            raw: *mut sdl3::sys::gamepad::SDL_Gamepad,
        }

        let gamepad_raw: &GamepadRaw = std::mem::transmute(pad);
        if gamepad_raw.raw.is_null() {
            tracing::warn!(
                "Gamepad raw pointer is null for instance ID {}",
                instance_id
            );
            return 0;
        }

        SDL_GetGamepadSteamHandle(gamepad_raw.raw)
    }
}

#[macro_export]
macro_rules! event_which {
    ($event:expr) => {
        match $event {
            Event::JoyAxisMotion { which, .. }
            // | Event::JoyBallMotion { which, .. }
            | Event::JoyHatMotion { which, .. }
            | Event::JoyButtonDown { which, .. }
            | Event::JoyButtonUp { which, .. }
            | Event::JoyDeviceAdded { which, .. }
            | Event::JoyDeviceRemoved { which, .. }
            // FUCK RUSTFMT
            | Event::ControllerAxisMotion { which, .. }
            | Event::ControllerButtonDown { which, .. }
            | Event::ControllerButtonUp { which, .. }
            | Event::ControllerDeviceAdded { which, .. }
            | Event::ControllerDeviceRemoved { which, .. }
            | Event::ControllerDeviceRemapped { which, .. }
            | Event::ControllerTouchpadDown { which, .. }
            | Event::ControllerTouchpadMotion { which, .. }
            | Event::ControllerTouchpadUp { which, .. }
            | Event::ControllerSensorUpdated { which, .. } => Some(*which),
            _ => None,
        }
    };
}
