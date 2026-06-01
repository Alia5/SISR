package gamepadupdated

import (
	"encoding"
	"math"

	"github.com/Alia5/SISR/sdl"
	"github.com/Alia5/VIIPER/device/ns2pro"
)

func toNS2ProState(gp *sdl.Gamepad, state *encoding.BinaryMarshaler) {
	s, ok := (*state).(*ns2pro.InputState)
	if !ok || s == nil {
		s = &ns2pro.InputState{}
		*state = s
	}
	s.Buttons = 0
	if gp.GetButton(sdl.GamepadButtonSouth) {
		s.Buttons |= ns2pro.ButtonB
	}
	if gp.GetButton(sdl.GamepadButtonEast) {
		s.Buttons |= ns2pro.ButtonA
	}
	if gp.GetButton(sdl.GamepadButtonWest) {
		s.Buttons |= ns2pro.ButtonY
	}
	if gp.GetButton(sdl.GamepadButtonNorth) {
		s.Buttons |= ns2pro.ButtonX
	}
	if gp.GetButton(sdl.GamepadButtonLeftShoulder) {
		s.Buttons |= ns2pro.ButtonL
	}
	if gp.GetButton(sdl.GamepadButtonRightShoulder) {
		s.Buttons |= ns2pro.ButtonR
	}
	if gp.GetButton(sdl.GamepadButtonLeftStick) {
		s.Buttons |= ns2pro.ButtonLeftStick
	}
	if gp.GetButton(sdl.GamepadButtonRightStick) {
		s.Buttons |= ns2pro.ButtonRightStick
	}
	if gp.GetButton(sdl.GamepadButtonStart) {
		s.Buttons |= ns2pro.ButtonPlus
	}
	if gp.GetButton(sdl.GamepadButtonBack) {
		s.Buttons |= ns2pro.ButtonMinus
	}
	if gp.GetButton(sdl.GamepadButtonGuide) {
		s.Buttons |= ns2pro.ButtonHome
	}
	if gp.GetButton(sdl.GamepadButtonDpadUp) {
		s.Buttons |= ns2pro.ButtonUp
	}
	if gp.GetButton(sdl.GamepadButtonDpadDown) {
		s.Buttons |= ns2pro.ButtonDown
	}
	if gp.GetButton(sdl.GamepadButtonDpadLeft) {
		s.Buttons |= ns2pro.ButtonLeft
	}
	if gp.GetButton(sdl.GamepadButtonDpadRight) {
		s.Buttons |= ns2pro.ButtonRight
	}

	lt := gp.GetAxis(sdl.GamepadAxisLeftTrigger)
	rt := gp.GetAxis(sdl.GamepadAxisRightTrigger)

	if lt > 128 {
		s.Buttons |= ns2pro.ButtonZL
	}
	if rt > 128 {
		s.Buttons |= ns2pro.ButtonZR
	}

	s.LX = uint16((int32(gp.GetAxis(sdl.GamepadAxisLeftX)) + 32768) >> 4)
	s.LY = uint16((int32(clamp(int(-int32(gp.GetAxis(sdl.GamepadAxisLeftY))), math.MinInt16, math.MaxInt16)) + 32768) >> 4)
	s.RX = uint16((int32(gp.GetAxis(sdl.GamepadAxisRightX)) + 32768) >> 4)
	s.RY = uint16((int32(clamp(int(-int32(gp.GetAxis(sdl.GamepadAxisRightY))), math.MinInt16, math.MaxInt16)) + 32768) >> 4)

}
