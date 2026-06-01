package touchpadupdated

import (
	"encoding"

	"github.com/Alia5/VIIPER/device/dualsense"
)

func updateTouchpadStateDualSense(slot int32, normX, normY float32, active bool, state *encoding.BinaryMarshaler) {
	s, ok := (*state).(*dualsense.InputState)
	if !ok || s == nil {
		s = &dualsense.InputState{}
		*state = s
	}

	switch slot {
	case 0:
		s.Touch1Active = active
		if active {
			s.Touch1X = uint16(normX * float32(dualsense.TouchpadMaxX))
			s.Touch1Y = uint16(normY * float32(dualsense.TouchpadMaxY))
		}
	case 1:
		s.Touch2Active = active
		if active {
			s.Touch2X = uint16(normX * float32(dualsense.TouchpadMaxX))
			s.Touch2Y = uint16(normY * float32(dualsense.TouchpadMaxY))
		}
	}
}
