package gamepadupdated

import (
	"encoding"
	"math"

	"github.com/Alia5/SISR/sdl"
	"github.com/Alia5/VIIPER/device/dualsense"
)

func toDualSenseState(gp *sdl.Gamepad, state *encoding.BinaryMarshaler) {
	s, ok := (*state).(*dualsense.InputState)
	if !ok || s == nil {
		s = &dualsense.InputState{}
		*state = s
	}
	s.Buttons = 0
	s.DPad = 0
	if gp.GetButton(sdl.GamepadButtonSouth) {
		s.Buttons |= dualsense.ButtonCross
	}
	if gp.GetButton(sdl.GamepadButtonEast) {
		s.Buttons |= dualsense.ButtonCircle
	}
	if gp.GetButton(sdl.GamepadButtonWest) {
		s.Buttons |= dualsense.ButtonSquare
	}
	if gp.GetButton(sdl.GamepadButtonNorth) {
		s.Buttons |= dualsense.ButtonTriangle
	}
	if gp.GetButton(sdl.GamepadButtonLeftShoulder) {
		s.Buttons |= dualsense.ButtonL1
	}
	if gp.GetButton(sdl.GamepadButtonRightShoulder) {
		s.Buttons |= dualsense.ButtonR1
	}
	if gp.GetButton(sdl.GamepadButtonLeftStick) {
		s.Buttons |= dualsense.ButtonL3
	}
	if gp.GetButton(sdl.GamepadButtonRightStick) {
		s.Buttons |= dualsense.ButtonR3
	}
	if gp.GetButton(sdl.GamepadButtonStart) {
		s.Buttons |= dualsense.ButtonOptions
	}
	if gp.GetButton(sdl.GamepadButtonBack) {
		s.Buttons |= dualsense.ButtonCreate
	}
	if gp.GetButton(sdl.GamepadButtonGuide) {
		s.Buttons |= dualsense.ButtonPS
	}
	if gp.GetButton(sdl.GamepadButtonDpadUp) {
		s.DPad |= dualsense.DPadUp
	}
	if gp.GetButton(sdl.GamepadButtonDpadDown) {
		s.DPad |= dualsense.DPadDown
	}
	if gp.GetButton(sdl.GamepadButtonDpadLeft) {
		s.DPad |= dualsense.DPadLeft
	}
	if gp.GetButton(sdl.GamepadButtonDpadRight) {
		s.DPad |= dualsense.DPadRight
	}

	lt := gp.GetAxis(sdl.GamepadAxisLeftTrigger)
	rt := gp.GetAxis(sdl.GamepadAxisRightTrigger)

	s.L2 = uint8(clamp(int((max(int32(0), int32(lt))*int32(math.MaxUint8))/int32(math.MaxInt16)), 0, math.MaxUint8))
	s.R2 = uint8(clamp(int((max(int32(0), int32(rt))*int32(math.MaxUint8))/int32(math.MaxInt16)), 0, math.MaxUint8))
	if lt > 128 {
		s.Buttons |= dualsense.ButtonL2
	}
	if rt > 128 {
		s.Buttons |= dualsense.ButtonR2
	}

	s.LX = int8(clamp(int((int32(gp.GetAxis(sdl.GamepadAxisLeftX))*int32(math.MaxInt8+1))/int32(math.MaxInt16)), math.MinInt8, math.MaxInt8))
	s.LY = int8(clamp(int((int32(gp.GetAxis(sdl.GamepadAxisLeftY))*int32(math.MaxInt8+1))/int32(math.MaxInt16)), math.MinInt8, math.MaxInt8))
	s.RX = int8(clamp(int((int32(gp.GetAxis(sdl.GamepadAxisRightX))*int32(math.MaxInt8+1))/int32(math.MaxInt16)), math.MinInt8, math.MaxInt8))
	s.RY = int8(clamp(int((int32(gp.GetAxis(sdl.GamepadAxisRightY))*int32(math.MaxInt8+1))/int32(math.MaxInt16)), math.MinInt8, math.MaxInt8))

	// TODO: backButtonPasthrough

}
