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
		return handleTouchpadEvent(c, ev, true, true)
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
		return handleTouchpadEvent(c, ev, true, false)
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
		return handleTouchpadEvent(c, ev, false, false)
	}
}

func handleTouchpadEvent(c *cmd.SISRContext, ev *sdl.GamepadTouchpadEvent, active, isDown bool) error {
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

	isDualTouchpad := dev.RealGamepad.NumTouchpads() >= 2

	var normX, normY float32
	if isDualTouchpad {
		normX = ev.X*0.5 + float32(ev.Touchpad)*0.5
		normY = ev.Y
	} else {
		normX = ev.X
		normY = ev.Y
	}

	dType := dev.ViiperDevice.Type()
	switch dType {
	case viiperdevice.DeviceTypeDualShock4:
		updateTouchpadStateDS4(ev.Touchpad, ev.Finger, normX, normY, active, isDown, isDualTouchpad, dev.ViiperDevice.State())
	case viiperdevice.DeviceTypeDualSense, viiperdevice.DeviceTypeDualSenseEdge:
		updateTouchpadStateDualSense(ev.Touchpad, ev.Finger, normX, normY, active, isDown, isDualTouchpad, dev.ViiperDevice.State())
	}

	return nil
}
