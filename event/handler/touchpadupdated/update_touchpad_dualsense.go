package touchpadupdated

import (
	"encoding"

	"github.com/Alia5/VIIPER/device/dualsense"
)

func updateTouchpadStateDualSense(pad, finger int32, normX, normY float32, active, isDown, isDualTouchpad bool, state *encoding.BinaryMarshaler) {
	s, ok := (*state).(*dualsense.InputState)
	if !ok || s == nil {
		s = &dualsense.InputState{}
		*state = s
	}

	if isDualTouchpad && pad == 1 {
		if !active {
			s.Touch2Active = false
			s.Touch1Active = false
		} else if isDown {
			if !s.Touch1Active {
				s.Touch1Active = true
				s.Touch1X = uint16(normX * float32(dualsense.TouchpadMaxX))
				s.Touch1Y = uint16(normY * float32(dualsense.TouchpadMaxY))
			} else {
				s.Touch2Active = true
				s.Touch2X = uint16(normX * float32(dualsense.TouchpadMaxX))
				s.Touch2Y = uint16(normY * float32(dualsense.TouchpadMaxY))
			}
		} else {
			if s.Touch2Active {
				s.Touch2X = uint16(normX * float32(dualsense.TouchpadMaxX))
				s.Touch2Y = uint16(normY * float32(dualsense.TouchpadMaxY))
			} else {
				s.Touch1X = uint16(normX * float32(dualsense.TouchpadMaxX))
				s.Touch1Y = uint16(normY * float32(dualsense.TouchpadMaxY))
			}
		}
		return
	}

	switch finger {
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
