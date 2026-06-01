package input

import (
	"log/slog"
	"math"
	"sync"

	"github.com/Alia5/SISR/input/viiperdevice"
	"github.com/Alia5/SISR/sdl"
	"github.com/Alia5/VIIPER/device/dualsense"
	"github.com/Alia5/VIIPER/device/dualshock4"
	"github.com/Alia5/VIIPER/device/keyboard"
	"github.com/Alia5/VIIPER/device/ns2pro"
	"github.com/Alia5/VIIPER/device/xbox360"
)

type Device struct {
	RealGamepad         *sdl.Gamepad
	SteamVirtualGamepad *sdl.Gamepad
	ViiperDevice        *viiperdevice.Device

	mtx sync.Mutex
}

func (d *Device) Lock() {
	d.mtx.Lock()
}

func (d *Device) Unlock() {
	d.mtx.Unlock()
}

func (d *Device) Gamepad(id sdl.GamepadID) (gp *sdl.Gamepad, isVirtual bool, err error) {
	if d.RealGamepad != nil && d.RealGamepad.ID() == id {
		return d.RealGamepad, false, nil
	}
	if d.SteamVirtualGamepad != nil && d.SteamVirtualGamepad.ID() == id {
		return d.SteamVirtualGamepad, true, nil
	}
	return gp, isVirtual, ErrNoDeviceForID
}

func (d *Device) Close() {
	if d.RealGamepad != nil {
		slog.Debug("Closing real gamepad", "id", d.RealGamepad.ID())
		d.RealGamepad.Close()
		d.RealGamepad = nil
	}
	if d.SteamVirtualGamepad != nil {
		slog.Debug("Closing steam virtual gamepad", "id", d.SteamVirtualGamepad.ID())
		d.SteamVirtualGamepad.Close()
		d.SteamVirtualGamepad = nil
	}
	if d.ViiperDevice != nil {
		slog.Debug("Closing VIIPER device", "info", d.ViiperDevice.Info())
		err := d.ViiperDevice.Close()
		if err != nil {
			slog.Error("Failed to close VIIPER device", "error", err)
		}
		d.ViiperDevice = nil
	}
}

func (d *Device) SetViiperDevice(vd *viiperdevice.Device) {
	if d.ViiperDevice != nil {
		slog.Warn("Device already has a VIIPER device assigned, Overwriting")
		err := d.ViiperDevice.Close()
		if err != nil {
			slog.Error("Failed to close existing VIIPER device when setting new one", "error", err)
		}
	}
	d.ViiperDevice = vd
	go d.handleFeedback(vd)
}

func (d *Device) handleFeedback(vd *viiperdevice.Device) {
	for {
		select {
		case fb := <-vd.FeedbackCh:
			if fb == nil {
				slog.Warn("Received nil feedback for VIIPER device; stopping feedback handling")
				return
			}
			switch fb := fb.(type) {
			case *xbox360.XRumbleState:
				d.handleXbox360Feedback(fb)
				continue
			case *dualshock4.OutputState:
				d.handleDualShock4Feedback(fb)
				continue
			case *keyboard.LEDState:
				d.handleKeyboardFeedback(fb)
				continue
			case *dualsense.OutputState:
				d.handleDualSenseFeedback(fb)
				continue
			case *ns2pro.OutputState:
				d.handleNS2ProFeedback(fb)
				continue
			default:
				slog.Warn("Received feedback of unknown type for VIIPER device; ignoring", "feedback", fb)
				continue
			}
		case e := <-vd.FeedbackErrCh:
			if e != nil {
				slog.Debug("feedback error", "error", e)
			}
			return
		case <-vd.DeviceCtx.Done():
			slog.Debug("VIIPER device context done, stopping feedback handling")
			return
		}
	}
}

func (d *Device) handleXbox360Feedback(rs *xbox360.XRumbleState) {
	d.mtx.Lock()
	defer d.mtx.Unlock()
	gp := d.SteamVirtualGamepad
	if gp == nil {
		slog.Warn("Received feedback for VIIPER device without a Steam virtual gamepad assigned; ignoring rumble command")
		return
	}
	_ = gp.Rumble(uint16(rs.LeftMotor)*257, uint16(rs.RightMotor)*257, math.MaxUint32)
}

func (d *Device) handleDualShock4Feedback(os *dualshock4.OutputState) {
	d.mtx.Lock()
	defer d.mtx.Unlock()
	gp := d.SteamVirtualGamepad
	if gp == nil {
		slog.Warn("Received feedback for VIIPER device without a Steam virtual gamepad assigned; ignoring rumble/led command")
		return
	}
	_ = gp.Rumble(uint16(os.RumbleLarge)*257, uint16(os.RumbleSmall)*257, math.MaxUint32)

	// TODO: handleFlashOnFlashOff
	rgp := d.RealGamepad
	if rgp != nil {
		props := rgp.GetProperties()
		switch {
		case props == 0:
		case sdl.GetBooleanProperty(props, sdl.PropGamepadCapRGBLEDBoolean(), false):
			_ = gp.SetLED(os.LedRed, os.LedGreen, os.LedBlue)
		case sdl.GetBooleanProperty(props, sdl.PropGamepadCapMonoLEDBoolean(), false):
			avg := (os.LedRed + os.LedGreen + os.LedBlue)
			_ = gp.SetLED(avg, avg, avg)
		}
	}
}

func (d *Device) handleKeyboardFeedback(ls *keyboard.LEDState) {
	// TODO: Implement keyboard LED feedback handling
}

func (d *Device) handleDualSenseFeedback(os *dualsense.OutputState) {

	slog.Debug("DualSense feedback received", "output_state", os)

	d.mtx.Lock()
	defer d.mtx.Unlock()
	gp := d.SteamVirtualGamepad
	if gp == nil {
		slog.Warn("Received feedback for VIIPER device without a Steam virtual gamepad assigned; ignoring rumble/led command")
		return
	}
	_ = gp.Rumble(uint16(os.RumbleLarge)*257, uint16(os.RumbleSmall)*257, math.MaxUint32)
}

func (d *Device) handleNS2ProFeedback(os *ns2pro.OutputState) {
	d.mtx.Lock()
	defer d.mtx.Unlock()
	gp := d.SteamVirtualGamepad
	if gp == nil {
		slog.Warn("Received feedback for VIIPER device without a Steam virtual gamepad assigned; ignoring rumble/led command")
		return
	}
	_ = gp.Rumble(uint16(os.LeftRumble[0])*257, uint16(os.RightRumble[0])*257, math.MaxUint32)
}
