package sisr

import (
	"log/slog"
	"runtime"
	"slices"

	"github.com/Alia5/SISR/windows/hooks"
)

var toUnhookFNs = []string{
	"HidD_FreePreparsedData",
	"HidD_GetAttributes",
	"HidD_GetPreparsedData",
	"HidD_GetProductString",
	"HidP_GetButtonCaps",
	"HidP_GetCaps",
	"HidP_GetData",
	"HidP_GetUsageValue",
	"HidP_GetUsages",
	"HidP_GetValueCaps",
	"HidP_MaxDataListLength",
}

func UnhookSteamHid() {
	if runtime.GOOS == "windows" {
		hookedFns := hooks.DetectHooks("hid.dll")
		if len(hookedFns) > 0 {
			slog.Info("Detected HID hooks")

			for _, toUnhook := range toUnhookFNs {
				if slices.Contains(hookedFns, toUnhook) {
					unhooked := hooks.Unhook("hid.dll", toUnhook)
					if unhooked {
						slog.Debug("Successfully unhooked HID export", "export", toUnhook)
					} else {
						slog.Warn("Failed to unhook HID export", "export", toUnhook)
					}
				} else {
					slog.Debug("HID export not hooked, skipping", "export", toUnhook)
				}
			}
		}
	}
}
