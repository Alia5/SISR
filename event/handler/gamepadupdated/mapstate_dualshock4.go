package gamepadupdated

import (
	"encoding"
	"math"

	"github.com/Alia5/SISR/sdl"
	"github.com/Alia5/VIIPER/device/dualshock4"
)

func toDualShock4State(gp *sdl.Gamepad, state *encoding.BinaryMarshaler) {
	s, ok := (*state).(*dualshock4.InputState)
	if !ok || s == nil {
		s = &dualshock4.InputState{}
		*state = s
	}
	s.Buttons = 0
	s.DPad = 0
	if gp.GetButton(sdl.GamepadButtonSouth) {
		s.Buttons |= dualshock4.ButtonCross
	}
	if gp.GetButton(sdl.GamepadButtonEast) {
		s.Buttons |= dualshock4.ButtonCircle
	}
	if gp.GetButton(sdl.GamepadButtonWest) {
		s.Buttons |= dualshock4.ButtonSquare
	}
	if gp.GetButton(sdl.GamepadButtonNorth) {
		s.Buttons |= dualshock4.ButtonTriangle
	}
	if gp.GetButton(sdl.GamepadButtonLeftShoulder) {
		s.Buttons |= dualshock4.ButtonL1
	}
	if gp.GetButton(sdl.GamepadButtonRightShoulder) {
		s.Buttons |= dualshock4.ButtonR1
	}
	if gp.GetButton(sdl.GamepadButtonLeftStick) {
		s.Buttons |= dualshock4.ButtonL3
	}
	if gp.GetButton(sdl.GamepadButtonRightStick) {
		s.Buttons |= dualshock4.ButtonR3
	}
	if gp.GetButton(sdl.GamepadButtonStart) {
		s.Buttons |= dualshock4.ButtonOptions
	}
	if gp.GetButton(sdl.GamepadButtonBack) {
		s.Buttons |= dualshock4.ButtonShare
	}
	if gp.GetButton(sdl.GamepadButtonGuide) {
		s.Buttons |= dualshock4.ButtonPS
	}
	if gp.GetButton(sdl.GamepadButtonDpadUp) {
		s.DPad |= dualshock4.DPadUp
	}
	if gp.GetButton(sdl.GamepadButtonDpadDown) {
		s.DPad |= dualshock4.DPadDown
	}
	if gp.GetButton(sdl.GamepadButtonDpadLeft) {
		s.DPad |= dualshock4.DPadLeft
	}
	if gp.GetButton(sdl.GamepadButtonDpadRight) {
		s.DPad |= dualshock4.DPadRight
	}

	lt := gp.GetAxis(sdl.GamepadAxisLeftTrigger)
	rt := gp.GetAxis(sdl.GamepadAxisRightTrigger)

	s.L2 = uint8(clamp(int((max(int32(0), int32(lt))*int32(math.MaxUint8))/int32(math.MaxInt16)), 0, math.MaxUint8))
	s.R2 = uint8(clamp(int((max(int32(0), int32(rt))*int32(math.MaxUint8))/int32(math.MaxInt16)), 0, math.MaxUint8))
	if lt > 128 {
		s.Buttons |= dualshock4.ButtonL2
	}
	if rt > 128 {
		s.Buttons |= dualshock4.ButtonR2
	}

	s.LX = int8(clamp(int((int32(gp.GetAxis(sdl.GamepadAxisLeftX))*int32(math.MaxInt8+1))/int32(math.MaxInt16)), math.MinInt8, math.MaxInt8))
	s.LY = int8(clamp(int((int32(gp.GetAxis(sdl.GamepadAxisLeftY))*int32(math.MaxInt8+1))/int32(math.MaxInt16)), math.MinInt8, math.MaxInt8))
	s.RX = int8(clamp(int((int32(gp.GetAxis(sdl.GamepadAxisRightX))*int32(math.MaxInt8+1))/int32(math.MaxInt16)), math.MinInt8, math.MaxInt8))
	s.RY = int8(clamp(int((int32(gp.GetAxis(sdl.GamepadAxisRightY))*int32(math.MaxInt8+1))/int32(math.MaxInt16)), math.MinInt8, math.MaxInt8))
}

func ds4TouchpadClickPassthrough(gp *sdl.Gamepad, state *encoding.BinaryMarshaler) {
	s, ok := (*state).(*dualshock4.InputState)
	if !ok || s == nil {
		return
	}

	if gp.GetButton(sdl.GamepadButtonTouchpad) {
		s.Buttons |= dualshock4.ButtonTouchpadClick
	} else {
		s.Buttons &^= dualshock4.ButtonTouchpadClick
	}
}
