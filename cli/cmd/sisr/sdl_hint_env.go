package sisr

import (
	"log/slog"
	"os"
)

var hintMap = map[string]string{
	"SteamStreamingVideo": "0",
	"SteamStreaming":      "0",
	"SDL_GAMECONTROLLER_ALLOW_STEAM_VIRTUAL_GAMEPAD": "1",
	"SDL_JOYSTICK_HIDAPI_STEAMXBOX":                  "1",
	// The SDL_GAMECONTROLLER_IGNORE_DEVICES hint doesn't work when Steam is
	// injected, but the env-var form does.
	"SDL_GAMECONTROLLER_IGNORE_DEVICES":        "",
	"SDL_GAMECONTROLLER_IGNORE_DEVICES_EXCEPT": "",
}

func SetSDLHintEnv() {
	for key, value := range hintMap {
		err := os.Setenv(key, value)
		if err != nil {
			slog.Error("Failed to set SDL hint environment variable", "key", key, "error", err)
		}
	}
}
