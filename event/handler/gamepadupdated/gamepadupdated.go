package gamepadupdated

import (
	"context"
	"log/slog"

	"github.com/Alia5/SISR/cmd"
	"github.com/Alia5/SISR/event/handler"
	"github.com/Alia5/SISR/input/viiperdevice"
	"github.com/Alia5/SISR/sdl"
)

func GamepadUpdated(c *cmd.SISRContext) handler.Operation[*sdl.GamepadDeviceEvent] {
	return handler.Operation[*sdl.GamepadDeviceEvent]{
		Event:   sdl.EventTypeGamepadUpdateComplete,
		Handler: handler.HandleFunc(gpUpdate(c)),
	}
}

func gpUpdate(c *cmd.SISRContext) func(ctx context.Context, ev *sdl.GamepadDeviceEvent) error {
	return func(ctx context.Context, ev *sdl.GamepadDeviceEvent) error {
		gpID := sdl.GamepadID(ev.Which)
		dev, ok := c.DeviceStore.DeviceForID(gpID)
		if !ok {
			return nil
		}
		dev.Lock()
		defer dev.Unlock()

		if dev.SteamVirtualGamepad == nil {
			return nil
		}

		if dev.SteamVirtualGamepad.ID() != gpID {

			c.Config.Lock()
			backButtonPassthrough := c.Config.BackButtonPassthrough
			touchpadPassthrough := c.Config.TouchpadPassthrough
			c.Config.Unlock()

			if dev.RealGamepad != nil && dev.RealGamepad.ID() == gpID &&
				dev.ViiperDevice != nil {
				dType := dev.ViiperDevice.Type()

				if backButtonPassthrough {
					switch dType {
					case viiperdevice.DeviceTypeDualSenseEdge:
						dualSenseBackButtonPassthrough(dev.RealGamepad, dev.ViiperDevice.State())
					case viiperdevice.DeviceTypeNS2Pro:
						ns2ProBackButtonPassthrough(dev.RealGamepad, dev.ViiperDevice.State())
					default:
					}
				}

				if touchpadPassthrough {
					switch dType {
					case viiperdevice.DeviceTypeDualShock4:
						ds4TouchpadClickPassthrough(dev.RealGamepad, dev.ViiperDevice.State())
					case viiperdevice.DeviceTypeDualSense, viiperdevice.DeviceTypeDualSenseEdge:
						dualSenseTouchpadClickPassthrough(dev.RealGamepad, dev.ViiperDevice.State())
					default:
					}
				}
			}

			return nil
		}

		if dev.ViiperDevice == nil {
			slog.Debug("No VIIPER device for gamepad found, scheduling create", "id", gpID)
			handler.CreateViiperDevice(ctx, c, gpID, dev)
			return nil
		}
		if dev.ViiperDevice.IsClosed() {
			slog.Info("VIIPER device is closed, cleaning up...", "id", gpID)
			// clear, exit, is recreated on next event
			dev.ViiperDevice = nil
			return nil
		}
		dType := dev.ViiperDevice.Type()

		switch dType {
		case viiperdevice.DeviceTypeXbox360:
			toXbox360State(dev.SteamVirtualGamepad, dev.ViiperDevice.State())
		case viiperdevice.DeviceTypeDualShock4:
			toDualShock4State(dev.SteamVirtualGamepad, dev.ViiperDevice.State())
		case viiperdevice.DeviceTypeDualSense, viiperdevice.DeviceTypeDualSenseEdge:
			toDualSenseState(dev.SteamVirtualGamepad, dev.ViiperDevice.State())
		case viiperdevice.DeviceTypeNS2Pro:
			toNS2ProState(dev.SteamVirtualGamepad, dev.ViiperDevice.State())
		default:
			slog.Warn("Cant update unknown VIIPER device type", "device_type", dType)
		}
		dev.ViiperDevice.QueueStateSend()

		return nil
	}
}
