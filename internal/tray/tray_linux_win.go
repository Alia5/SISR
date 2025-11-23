//go:build linux || windows

package tray

import (
	"fyne.io/systray"
	fyneIcon "fyne.io/systray/example/icon"
)

func setup() {
	systray.SetIcon(fyneIcon.Data)
	systray.SetTitle("SISR")
	systray.SetTooltip("Steam Input System Redirector")
}
