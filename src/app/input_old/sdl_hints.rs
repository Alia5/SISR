use sdl3::hint;

pub const SDL_HINTS: &[(&str, &str)] = &[
    (hint::names::JOYSTICK_ALLOW_BACKGROUND_EVENTS, "1"),
    (hint::names::HIDAPI_IGNORE_DEVICES, ""),
    (hint::names::GAMECONTROLLER_IGNORE_DEVICES, ""),
    (hint::names::HIDAPI_UDEV, "1"),
    (hint::names::JOYSTICK_HIDAPI, "1"),
    // TODO: check
];
