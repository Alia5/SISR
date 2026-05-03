use sdl3::hint;

pub const SDL_HINTS: &[(&str, &str)] = &[
    (hint::names::JOYSTICK_ALLOW_BACKGROUND_EVENTS, "1"),
    (hint::names::HIDAPI_IGNORE_DEVICES, ""),
    (hint::names::GAMECONTROLLER_IGNORE_DEVICES, ""),
    (hint::names::HIDAPI_UDEV, "1"),
    (hint::names::XINPUT_ENABLED, "1"),
    // (hint::names::JOYSTICK_)
    // (hint::names::JOYSTICK_RAWINPUT, "1"),
    // (hint::names::JOYSTICK_RAWINPUT_CORRELATE_XINPUT, "1"),
    (hint::names::JOYSTICK_HIDAPI, "1"),
    (hint::names::JOYSTICK_HIDAPI_SWITCH, "1"),
    (hint::names::JOYSTICK_HIDAPI_SWITCH2, "1"),
    (hint::names::JOYSTICK_HIDAPI_JOY_CONS, "1"),
    (hint::names::JOYSTICK_HIDAPI_NINTENDO_CLASSIC, "1"),
    // (hint::names::JOYSTICK_HIDAPI_COMBINE_JOY_CONS, "1"),
    (hint::names::JOYSTICK_HIDAPI_XBOX, "0"), // causes crash for some odd reason when enabled?
    (hint::names::JOYSTICK_HIDAPI_XBOX_360, "0"), // causes crash for some odd reason when enabled?
    (hint::names::JOYSTICK_HIDAPI_XBOX_ONE, "0"), // causes crash for some odd reason when enabled?
    // (hint::names::JOYSTICK_RAWINPUT, "1"),
    // (hint::names::JOYSTICK_RAWINPUT_CORRELATE_XINPUT, "1"),
    (hint::names::JOYSTICK_ENHANCED_REPORTS, "1"),
    // (hint::names::JOYSTICK_DIRECTINPUT, "1"),
    // (hint::names::JOYSTICK_GAMEINPUT, "1"),
    //(hint::names::HIDAPI_LIBUSB, "1"),
];
