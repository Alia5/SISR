package sensorupdated

import (
	"encoding"
	"math"

	"github.com/Alia5/SISR/sdl"
	"github.com/Alia5/VIIPER/device/dualsense"
)

func updateSensorStateDualSense(sensorType sdl.SensorType, data [3]float32, state *encoding.BinaryMarshaler) {
	s, ok := (*state).(*dualsense.InputState)
	if !ok || s == nil {
		s = &dualsense.InputState{}
		*state = s
	}

	switch sensorType {
	case sdl.SensorTypeGyroscope:
		// SDL3 provides gyroscope data in rad/s
		// See: https://github.com/libsdl-org/SDL/blob/main/include/SDL3/SDL_sensor.h
		// VIIPER DS5 input expects fixed-point °/s
		// See: https://alia5.github.io/VIIPER/main/devices/dualsense/
		s.GyroX = int16(math.Round(min(float64(math.MaxInt16), max(float64(math.MinInt16), float64(data[0]*(180.0/math.Pi)*dualsense.GyroCountsPerDps)))))
		s.GyroY = int16(math.Round(min(float64(math.MaxInt16), max(float64(math.MinInt16), float64(data[1]*(180.0/math.Pi)*dualsense.GyroCountsPerDps)))))
		s.GyroZ = int16(math.Round(min(float64(math.MaxInt16), max(float64(math.MinInt16), float64(data[2]*(180.0/math.Pi)*dualsense.GyroCountsPerDps)))))
	case sdl.SensorTypeAccelerometer:
		s.AccelX = int16(math.Round(min(float64(math.MaxInt16), max(float64(math.MinInt16), float64(data[0]*dualsense.AccelCountsPerMS2)))))
		s.AccelY = int16(math.Round(min(float64(math.MaxInt16), max(float64(math.MinInt16), float64(data[1]*dualsense.AccelCountsPerMS2)))))
		s.AccelZ = int16(math.Round(min(float64(math.MaxInt16), max(float64(math.MinInt16), float64(data[2]*dualsense.AccelCountsPerMS2)))))
	}

}
