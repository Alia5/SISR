package touchpadupdated

import (
	"encoding"

	"github.com/Alia5/VIIPER/device/dualshock4"
)

func updateTouchpadStateDS4(pad, finger int32, normX, normY float32, active, isDown, isDualTouchpad bool, state *encoding.BinaryMarshaler) {
	s, ok := (*state).(*dualshock4.InputState)
	if !ok || s == nil {
		s = &dualshock4.InputState{}
		*state = s
	}

	if isDualTouchpad && pad == 1 {
		if !active {
			s.Touch2Active = false
			s.Touch1Active = false
		} else if isDown {
			if !s.Touch1Active {
				s.Touch1Active = true
				s.Touch1X = uint16(normX * float32(dualshock4.TouchpadMaxX))
				s.Touch1Y = uint16(normY * float32(dualshock4.TouchpadMaxY))
			} else {
				s.Touch2Active = true
				s.Touch2X = uint16(normX * float32(dualshock4.TouchpadMaxX))
				s.Touch2Y = uint16(normY * float32(dualshock4.TouchpadMaxY))
			}
		} else {
			if s.Touch2Active {
				s.Touch2X = uint16(normX * float32(dualshock4.TouchpadMaxX))
				s.Touch2Y = uint16(normY * float32(dualshock4.TouchpadMaxY))
			} else {
				s.Touch1X = uint16(normX * float32(dualshock4.TouchpadMaxX))
				s.Touch1Y = uint16(normY * float32(dualshock4.TouchpadMaxY))
			}
		}
		return
	}

	switch finger {
	case 0:
		s.Touch1Active = active
		if active {
			s.Touch1X = uint16(normX * float32(dualshock4.TouchpadMaxX))
			s.Touch1Y = uint16(normY * float32(dualshock4.TouchpadMaxY))
		}
	case 1:
		s.Touch2Active = active
		if active {
			s.Touch2X = uint16(normX * float32(dualshock4.TouchpadMaxX))
			s.Touch2Y = uint16(normY * float32(dualshock4.TouchpadMaxY))
		}
	}
}
