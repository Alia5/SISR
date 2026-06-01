package touchpadupdated

import (
	"context"

	"github.com/Alia5/SISR/cmd"
	"github.com/Alia5/SISR/event/handler"
	"github.com/Alia5/SISR/input/viiperdevice"
	"github.com/Alia5/SISR/sdl"
)

func TouchpadDown(c *cmd.SISRContext) handler.Operation[*sdl.GamepadTouchpadEvent] {
	return handler.Operation[*sdl.GamepadTouchpadEvent]{
		Event:   sdl.EventTypeGamepadTouchpadDown,
		Handler: handler.HandleFunc(touchpadDown(c)),
	}
}

func TouchpadMotion(c *cmd.SISRContext) handler.Operation[*sdl.GamepadTouchpadEvent] {
	return handler.Operation[*sdl.GamepadTouchpadEvent]{
		Event:   sdl.EventTypeGamepadTouchpadMotion,
		Handler: handler.HandleFunc(touchpadMotion(c)),
	}
}

func TouchpadUp(c *cmd.SISRContext) handler.Operation[*sdl.GamepadTouchpadEvent] {
	return handler.Operation[*sdl.GamepadTouchpadEvent]{
		Event:   sdl.EventTypeGamepadTouchpadUp,
		Handler: handler.HandleFunc(touchpadUp(c)),
	}
}

func touchpadDown(c *cmd.SISRContext) func(ctx context.Context, ev *sdl.GamepadTouchpadEvent) error {
	c.Config.Lock()
	touchpadPassthrough := c.Config.TouchpadPassthrough
	c.Config.Unlock()

	return func(ctx context.Context, ev *sdl.GamepadTouchpadEvent) error {
		if !touchpadPassthrough {
			return nil
		}
		return handleTouchpadEvent(c, ev, true)
	}
}

func touchpadMotion(c *cmd.SISRContext) func(ctx context.Context, ev *sdl.GamepadTouchpadEvent) error {
	c.Config.Lock()
	touchpadPassthrough := c.Config.TouchpadPassthrough
	c.Config.Unlock()

	return func(ctx context.Context, ev *sdl.GamepadTouchpadEvent) error {
		if !touchpadPassthrough {
			return nil
		}
		return handleTouchpadEvent(c, ev, true)
	}
}

func touchpadUp(c *cmd.SISRContext) func(ctx context.Context, ev *sdl.GamepadTouchpadEvent) error {
	c.Config.Lock()
	touchpadPassthrough := c.Config.TouchpadPassthrough
	c.Config.Unlock()

	return func(ctx context.Context, ev *sdl.GamepadTouchpadEvent) error {
		if !touchpadPassthrough {
			return nil
		}
		return handleTouchpadEvent(c, ev, false)
	}
}

func handleTouchpadEvent(c *cmd.SISRContext, ev *sdl.GamepadTouchpadEvent, active bool) error {
	gpID := sdl.GamepadID(ev.Which)
	dev, ok := c.DeviceStore.DeviceForID(gpID)
	if !ok {
		return nil
	}
	dev.Lock()
	defer dev.Unlock()

	if dev.SteamVirtualGamepad == nil || dev.RealGamepad == nil {
		return nil
	}
	if dev.RealGamepad.ID() != gpID {
		return nil
	}

	if dev.ViiperDevice == nil {
		return nil
	}
	if dev.ViiperDevice.IsClosed() {
		return nil
	}

	slot, normX, normY := resolveTouchPoint(ev, dev.RealGamepad.NumTouchpads())

	dType := dev.ViiperDevice.Type()
	switch dType {
	case viiperdevice.DeviceTypeDualShock4:
		updateTouchpadStateDS4(slot, normX, normY, active, dev.ViiperDevice.State())
	case viiperdevice.DeviceTypeDualSense, viiperdevice.DeviceTypeDualSenseEdge:
		updateTouchpadStateDualSense(slot, normX, normY, active, dev.ViiperDevice.State())
	}

	return nil
}

func resolveTouchPoint(ev *sdl.GamepadTouchpadEvent, numTouchpads int) (slot int32, normX, normY float32) {
	if numTouchpads >= 2 {
		normX = ev.X * 0.5
		if ev.Touchpad == 1 {
			normX += 0.5
		}
		return ev.Touchpad, normX, ev.Y
	}
	return ev.Finger, ev.X, ev.Y
}
