package kbmforward

import (
	"context"
	"log/slog"

	"github.com/Alia5/SISR/cmd"
	"github.com/Alia5/SISR/event/handler"
	"github.com/Alia5/SISR/sdl"
	"github.com/Alia5/VIIPER/device/mouse"
)

func KeyboardDown(c *cmd.SISRContext, devices *handler.KBMDevices) handler.Operation[*sdl.KeyboardEvent] {
	return handler.Operation[*sdl.KeyboardEvent]{
		Event: sdl.EventTypeKeyDown,
		Handler: handler.HandleFunc(func(ctx context.Context, ev *sdl.KeyboardEvent) error {
			devices.Lock()
			defer devices.Unlock()

			if ev.Repeat {
				return nil
			}

			if devices.KeyboardDev == nil {
				handler.CreateViiperKBMDevice(ctx, c, "keyboard", devices)
				return nil
			}

			state := devices.KeyboardDev.EnsureKeyboardState()
			mapKeyboardStateDown(state, ev)
			devices.KeyboardDev.QueueStateSend()
			return nil
		}),
	}
}

func KeyboardUp(c *cmd.SISRContext, devices *handler.KBMDevices) handler.Operation[*sdl.KeyboardEvent] {
	return handler.Operation[*sdl.KeyboardEvent]{
		Event: sdl.EventTypeKeyUp,
		Handler: handler.HandleFunc(func(ctx context.Context, ev *sdl.KeyboardEvent) error {
			devices.Lock()
			defer devices.Unlock()
			if devices.KeyboardDev == nil {
				return nil
			}

			state := devices.KeyboardDev.EnsureKeyboardState()
			mapKeyboardStateUp(state, ev)
			devices.KeyboardDev.QueueStateSend()
			return nil
		}),
	}
}

func MouseMotion(c *cmd.SISRContext, devices *handler.KBMDevices) handler.Operation[*sdl.MouseMotionEvent] {
	return handler.Operation[*sdl.MouseMotionEvent]{
		Event: sdl.EventTypeMouseMotion,
		Handler: handler.HandleFunc(func(ctx context.Context, ev *sdl.MouseMotionEvent) error {
			devices.Lock()
			defer devices.Unlock()

			if devices.MouseDev == nil {
				handler.CreateViiperKBMDevice(ctx, c, "mouse", devices)
				return nil
			}

			state := devices.MouseDev.EnsureMouseState()
			state.DX = int16(ev.XRel)
			state.DY = int16(ev.YRel)
			devices.MouseDev.QueueStateSend()
			return nil
		}),
	}
}

func MouseButtonDown(c *cmd.SISRContext, devices *handler.KBMDevices) handler.Operation[*sdl.MouseButtonEvent] {
	return handler.Operation[*sdl.MouseButtonEvent]{
		Event: sdl.EventTypeMouseButtonDown,
		Handler: handler.HandleFunc(func(ctx context.Context, ev *sdl.MouseButtonEvent) error {
			devices.Lock()
			defer devices.Unlock()

			if devices.MouseDev == nil {
				handler.CreateViiperKBMDevice(ctx, c, "mouse", devices)
				return nil
			}

			state := devices.MouseDev.EnsureMouseState()
			switch ev.Button {
			case 1:
				state.Buttons |= mouse.BtnLeft
			case 2:
				state.Buttons |= mouse.BtnRight
			case 3:
				state.Buttons |= mouse.BtnMiddle
			case 4:
				state.Buttons |= mouse.BtnBack
			case 5:
				state.Buttons |= mouse.BtnForward
			}
			devices.MouseDev.QueueStateSend()
			return nil
		}),
	}
}

func MouseButtonUp(c *cmd.SISRContext, devices *handler.KBMDevices) handler.Operation[*sdl.MouseButtonEvent] {
	return handler.Operation[*sdl.MouseButtonEvent]{
		Event: sdl.EventTypeMouseButtonUp,
		Handler: handler.HandleFunc(func(ctx context.Context, ev *sdl.MouseButtonEvent) error {
			devices.Lock()
			defer devices.Unlock()

			if devices.MouseDev == nil {
				return nil
			}

			state := devices.MouseDev.EnsureMouseState()
			switch ev.Button {
			case 1:
				state.Buttons &^= mouse.BtnLeft
			case 2:
				state.Buttons &^= mouse.BtnRight
			case 3:
				state.Buttons &^= mouse.BtnMiddle
			case 4:
				state.Buttons &^= mouse.BtnBack
			case 5:
				state.Buttons &^= mouse.BtnForward
			}
			devices.MouseDev.QueueStateSend()
			return nil
		}),
	}
}

func MouseWheel(c *cmd.SISRContext, devices *handler.KBMDevices) handler.Operation[*sdl.MouseWheelEvent] {
	return handler.Operation[*sdl.MouseWheelEvent]{
		Event: sdl.EventTypeMouseWheel,
		Handler: handler.HandleFunc(func(ctx context.Context, ev *sdl.MouseWheelEvent) error {
			devices.Lock()
			defer devices.Unlock()
			if devices.MouseDev == nil {
				handler.CreateViiperKBMDevice(ctx, c, "mouse", devices)
				return nil
			}

			state := devices.MouseDev.EnsureMouseState()
			state.Wheel = int16(ev.IntegerY)
			state.Pan = int16(ev.IntegerX)
			devices.MouseDev.QueueStateSend()
			return nil
		}),
	}
}

func WindowFocusLost(c *cmd.SISRContext, window *sdl.Window, devices *handler.KBMDevices) handler.Operation[*sdl.WindowEvent] {
	return handler.Operation[*sdl.WindowEvent]{
		Event: sdl.EventTypeWindowFocusLost,
		Handler: handler.HandleFunc(func(ctx context.Context, ev *sdl.WindowEvent) error {
			devices.Lock()
			defer devices.Unlock()
			if window != nil {
				if err := window.SetWindowMouseGrab(false); err != nil {
					slog.Error("Failed to release window mouse grab on focus lost", "error", err)
				}
				if err := window.SetWindowRelativeMouseMode(false); err != nil {
					slog.Error("Failed to disable relative mouse mode on focus lost", "error", err)
				}
			}
			if devices.KeyboardDev != nil {
				keyboardState := devices.KeyboardDev.EnsureKeyboardState()
				keyboardState.Modifiers = 0
				clear(keyboardState.KeyBitmap[:])
				devices.KeyboardDev.QueueStateSend()
			}
			if devices.MouseDev == nil {
				return nil
			}
			state := devices.MouseDev.EnsureMouseState()
			state.Buttons = 0
			state.DX = 0
			state.DY = 0
			state.Wheel = 0
			state.Pan = 0
			devices.MouseDev.QueueStateSend()
			return nil
		}),
	}
}

func WindowFocusGained(c *cmd.SISRContext, window *sdl.Window, devices *handler.KBMDevices) handler.Operation[*sdl.WindowEvent] {
	return handler.Operation[*sdl.WindowEvent]{
		Event: sdl.EventTypeWindowFocusGained,
		Handler: handler.HandleFunc(func(ctx context.Context, ev *sdl.WindowEvent) error {
			if window != nil {
				if err := window.SetWindowMouseGrab(true); err != nil {
					slog.Error("Failed to enable window mouse grab on focus gained", "error", err)
				}
				if err := window.SetWindowRelativeMouseMode(true); err != nil {
					slog.Error("Failed to enable relative mouse mode on focus gained", "error", err)
				}
			}
			return nil
		}),
	}
}

func WindowHidden(c *cmd.SISRContext, window *sdl.Window, devices *handler.KBMDevices) handler.Operation[*sdl.WindowEvent] {
	return handler.Operation[*sdl.WindowEvent]{
		Event: sdl.EventTypeWindowHidden,
		Handler: handler.HandleFunc(func(ctx context.Context, ev *sdl.WindowEvent) error {
			devices.Lock()
			defer devices.Unlock()
			if window != nil {
				if err := window.SetWindowMouseGrab(false); err != nil {
					slog.Error("Failed to release window mouse grab on window hidden", "error", err)
				}
				if err := window.SetWindowRelativeMouseMode(false); err != nil {
					slog.Error("Failed to disable relative mouse mode on window hidden", "error", err)
				}
			}
			if devices.KeyboardDev != nil {
				keyboardState := devices.KeyboardDev.EnsureKeyboardState()
				keyboardState.Modifiers = 0
				clear(keyboardState.KeyBitmap[:])
				devices.KeyboardDev.QueueStateSend()
			}
			if devices.MouseDev == nil {
				return nil
			}
			state := devices.MouseDev.EnsureMouseState()
			state.Buttons = 0
			state.DX = 0
			state.DY = 0
			state.Wheel = 0
			state.Pan = 0
			devices.MouseDev.QueueStateSend()
			return nil
		}),
	}
}
