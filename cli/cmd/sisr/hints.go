package sisr

import (
	"log/slog"

	"github.com/Alia5/SISR/sdl"
)

var sdlHints = map[sdl.Hint]string{
	sdl.HintJoystickAllowBackgroundEvents: "1",
	sdl.HintHIDAPIIgnoreDevices:           "",
	// sdl.HintGamecontrollerIgnoreDevices:   "", // Set by env, this hint is blocked by Steam, but not from env...?
	sdl.HintHIDAPIUdev:    "1",
	sdl.HintXInputEnabled: "1",
	//
	sdl.HintJoystickHIDAPI: "1",
	//
	sdl.HintJoystickEnhancedReports: "1",

	sdl.HintJoystickHIDAPISwitch:          "1",
	sdl.HintJoystickHIDAPISwitch2:         "1",
	sdl.HintJoystickHIDAPIJoyCons:         "1",
	sdl.HintJoystickHIDAPINintendoClassic: "1",

	sdl.HintJoystickHIDAPIPS3: "1",
	sdl.HintJoystickHIDAPIPS4: "1",
	sdl.HintJoystickHIDAPIPS5: "1",
	// sdl.HintJoystickHIDAPICombineJoyCons: "1",
	sdl.HintJoystickHIDAPIXbox:            "1",
	sdl.HintJoystickHIDAPIXbox360:         "1",
	sdl.HintJoystickHIDAPIXbox360Wireless: "1",
	sdl.HintJoystickHIDAPIXboxOne:         "1",
	sdl.HintJoystickHIDAPIGIP:             "1",

	sdl.HintJoystickWGI:       "0",
	sdl.HintJoystickGameinput: "1",

	// sdl.HintHIDAPILibUSB:             "1",
}

func SetSDLHints() {
	for hint, value := range sdlHints {
		err := sdl.SetHint(hint, value)
		if err != nil {
			slog.Error("Failed to set SDL hint", "hint", hint, "value", value, "error", err)
		}
	}
}
