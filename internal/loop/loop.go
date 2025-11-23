package loop

import (
	"context"
	"log/slog"

	"github.com/Alia5/SISR/internal/tray"
	"github.com/Alia5/SISR/internal/viiper"
	"github.com/Zyko0/go-sdl3/sdl"
)

type Loop struct {
	ctx       context.Context
	trayChann chan tray.Signal
	vc        *viiper.Client
}

func New(ctx context.Context, trayChann chan tray.Signal, vc *viiper.Client) *Loop {
	return &Loop{
		ctx:       ctx,
		trayChann: trayChann,
		vc:        vc,
	}
}

func (l *Loop) Run(w *sdl.Window, r *sdl.Renderer) error {
	return sdl.RunLoop(func() error {
		var event sdl.Event

		for sdl.PollEvent(&event) {
			switch event.Type {
			case sdl.EVENT_QUIT:
				return sdl.EndLoop
			case sdl.EVENT_GAMEPAD_ADDED:
				l.onGamepadAdded(event.GamepadDeviceEvent())
			case sdl.EVENT_GAMEPAD_REMOVED:
				l.onGamepadRemoved(event.GamepadDeviceEvent())
			case sdl.EVENT_GAMEPAD_UPDATE_COMPLETE:
				l.onGamepadUpdate()
			}
		}
		if w != nil && r != nil {
			r.Clear()
			r.Present()
		}

		select {
		case traySig := <-l.trayChann:
			switch traySig {
			case tray.SignalQuit, tray.SignalExit:
				return sdl.EndLoop
			}
		case <-l.ctx.Done():
			return l.ctx.Err()
		default:
		}
		return nil
	})
}

func (l *Loop) onGamepadAdded(gpEv *sdl.GamepadDeviceEvent) {
	slog.Debug("Gamepad connected")
	if gpEv == nil || !gpEv.Which.IsGamepad() {
		return
	}
	name, _ := gpEv.Which.GamepadName()
	padType := gpEv.Which.GamepadType()
	padTypeString := padType.GamepadStringForType()
	isVirtual := gpEv.Which.IsJoystickVirtual()
	pad, err := gpEv.Which.OpenGamepad()
	if err != nil {
		slog.Error("Failed to open gamepad", "error", err)
		return
	}
	steamHandle := pad.SteamHandle()
	slog.Debug("Gamepad details", "Name", name, "Type", padTypeString, "Virtual", isVirtual, "SteamHandle", steamHandle)
	if l.vc != nil {
		if steamHandle == 0 {
			slog.Debug("Skipping VIIPER registration (no Steam handle)")
			pad.Close()
			return
		}
		if err := l.vc.AddGamepad(pad); err != nil {
			slog.Error("Failed to add VIIPER gamepad", "error", err)
			pad.Close()
			return
		}
		// Keep pad open (tracked by viiper client)
	} else {
		pad.Close()
	}
}

func (l *Loop) onGamepadRemoved(gpEv *sdl.GamepadDeviceEvent) {
	slog.Debug("Gamepad disconnected")
	if gpEv == nil || !gpEv.Which.IsGamepad() {
		return
	}
	pad, err := gpEv.Which.Gamepad()
	if err != nil || pad == nil {
		return
	}
	id, err := pad.ID()
	if err == nil && l.vc != nil {
		if err := l.vc.RemoveGamepad(id); err != nil {
			slog.Error("Failed to remove VIIPER gamepad", "error", err)
		}
	}
	pad.Close()
}

func (l *Loop) onGamepadUpdate() {
	if l.vc == nil {
		return
	}
	if err := l.vc.UpdateAllGamepads(); err != nil {
		slog.Error("Failed to update VIIPER gamepads", "error", err)
	}
}
