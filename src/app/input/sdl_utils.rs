use sdl3::gamepad::Gamepad;
use sdl3_sys::joystick::SDL_JoystickID;

pub fn get_gamepad_steam_handle(pad: &Gamepad) -> u64 {
    use sdl3::sys::gamepad::SDL_GetGamepadSteamHandle;
    let instance_id = pad.id().unwrap_or(SDL_JoystickID(0)).0;
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

pub fn try_get_real_vid_pid_from_gamepad(gp: &sdl3::gamepad::Gamepad) -> Option<(String, String)> {
    let vid = gp.vendor_id();
    let pid = gp.product_id();

    let mut fallback = None;
    if let Some(vid) = vid
        && let Some(pid) = pid
    {
        fallback = Some((
            format!("0x{:04x}", vid).to_lowercase(),
            format!("0x{:04x}", pid).to_lowercase(),
        ));
    }

    // Path: \\.\pipe\HID#VID_045E&PID_028E&IG_00#045E&028E&00645E28E235E61F#2#4828
    //       \\?\HID#VID_28DE&PID_1102&MI_02#a&35874d6&0&0000#{4d1e55b2-f16f-11cf-88cb-001111000030}                                   ^^^^ ^^^^
    let Some(path) = gp.path() else {
        return fallback;
    };

    if path.contains("HID#") {
        let parts: Vec<&str> = path.split('#').collect();
        if parts.len() >= 2 {
            // parts[1] should be "VID_045E&PID_028E&IG_00"
            let vid_pid_part = parts[1];
            let vid_pid: Vec<&str> = vid_pid_part.split('&').collect();
            let mut vid_opt = None;
            let mut pid_opt = None;
            for part in vid_pid {
                if part.starts_with("VID_") {
                    vid_opt = Some(part.trim_start_matches("VID_"));
                } else if part.starts_with("PID_") {
                    pid_opt = Some(part.trim_start_matches("PID_"));
                }
            }
            if let (Some(vid), Some(pid)) = (vid_opt, pid_opt) {
                return Some((
                    format!("0x{}", vid).to_lowercase(),
                    format!("0x{}", pid).to_lowercase(),
                ));
            }
        }
    } else {
        let parts: Vec<&str> = path.split('#').collect();
        if parts.len() >= 3 {
            // parts[2] should be "045E&028E&00645E28E235E61F"
            let real_device = parts[2];
            let vid_pid: Vec<&str> = real_device.split('&').collect();
            if vid_pid.len() >= 2 {
                return Some((
                    format!("0x{}", vid_pid[0]).to_lowercase(),
                    format!("0x{}", vid_pid[1]).to_lowercase(),
                ));
            }
            return fallback;
        }
    }

    fallback
}
