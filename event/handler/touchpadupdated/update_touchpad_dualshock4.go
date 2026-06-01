package touchpadupdated

import (
	"encoding"

	"github.com/Alia5/VIIPER/device/dualshock4"
)

func updateTouchpadStateDS4(slot int32, normX, normY float32, active bool, state *encoding.BinaryMarshaler) {
	s, ok := (*state).(*dualshock4.InputState)
	if !ok || s == nil {
		s = &dualshock4.InputState{}
		*state = s
	}

	switch slot {
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
