package sensorupdated

import (
	"context"
	"log/slog"

	"github.com/Alia5/SISR/cmd"
	"github.com/Alia5/SISR/event/handler"
	"github.com/Alia5/SISR/input/viiperdevice"
	"github.com/Alia5/SISR/sdl"
)

func SensorUpdated(c *cmd.SISRContext) handler.Operation[*sdl.GamepadSensorEvent] {
	return handler.Operation[*sdl.GamepadSensorEvent]{
		Event:   sdl.EventTypeGamepadSensorUpdate,
		Handler: handler.HandleFunc(sensorUpdate(c)),
	}
}

func sensorUpdate(c *cmd.SISRContext) func(ctx context.Context, ev *sdl.GamepadSensorEvent) error {

	c.Config.Lock()
	immediateSensorUpdates := c.Config.ImmediateSensorUpdates
	c.Config.Unlock()

	return func(ctx context.Context, ev *sdl.GamepadSensorEvent) error {
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

		updated := false
		dType := dev.ViiperDevice.Type()
		switch dType {
		case viiperdevice.DeviceTypeXbox360:
			// no gyro to update
		case viiperdevice.DeviceTypeDualShock4:
			sensorType := sdl.SensorType(ev.Sensor)
			if sensorType == sdl.SensorTypeGyroscope || sensorType == sdl.SensorTypeAccelerometer {
				updateSensorStateDS4(sensorType, [3]float32{ev.Data0, ev.Data1, ev.Data2}, dev.ViiperDevice.State())
				updated = true
			}
		case viiperdevice.DeviceTypeDualSense, viiperdevice.DeviceTypeDualSenseEdge:
			sensorType := sdl.SensorType(ev.Sensor)
			if sensorType == sdl.SensorTypeGyroscope || sensorType == sdl.SensorTypeAccelerometer {
				updateSensorStateDualSense(sensorType, [3]float32{ev.Data0, ev.Data1, ev.Data2}, dev.ViiperDevice.State())
				updated = true
			}
		case viiperdevice.DeviceTypeNS2Pro:
			sensorType := sdl.SensorType(ev.Sensor)
			if sensorType == sdl.SensorTypeGyroscope || sensorType == sdl.SensorTypeAccelerometer {
				updateSensorStateNS2Pro(sensorType, [3]float32{ev.Data0, ev.Data1, ev.Data2}, dev.ViiperDevice.State())
				updated = true
			}
		default:
			slog.Warn("Sensors: Cant update unknown VIIPER device type", "device_type", dType)
		}

		if updated && immediateSensorUpdates {
			dev.ViiperDevice.QueueStateSend()
		}

		return nil
	}
}
