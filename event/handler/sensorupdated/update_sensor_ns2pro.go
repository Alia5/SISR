package sensorupdated

import (
	"encoding"
	"math"

	"github.com/Alia5/SISR/sdl"
	"github.com/Alia5/VIIPER/device/ns2pro"
)

func updateSensorStateNS2Pro(sensorType sdl.SensorType, data [3]float32, state *encoding.BinaryMarshaler) {
	s, ok := (*state).(*ns2pro.InputState)
	if !ok || s == nil {
		s = &ns2pro.InputState{}
		*state = s
	}

	switch sensorType {
	case sdl.SensorTypeGyroscope:
		const gyroScale = (180.0 / math.Pi) * 16.384
		s.GyroX = int16(math.Round(min(float64(math.MaxInt16), max(float64(math.MinInt16), float64(data[0])*gyroScale))))
		s.GyroY = int16(math.Round(min(float64(math.MaxInt16), max(float64(math.MinInt16), float64(-data[2])*gyroScale))))
		s.GyroZ = int16(math.Round(min(float64(math.MaxInt16), max(float64(math.MinInt16), float64(data[1])*gyroScale))))
	case sdl.SensorTypeAccelerometer:
		const accelScale = 4096.0 / 9.80665
		s.AccelX = int16(math.Round(min(float64(math.MaxInt16), max(float64(math.MinInt16), float64(data[0])*accelScale))))
		s.AccelY = int16(math.Round(min(float64(math.MaxInt16), max(float64(math.MinInt16), float64(-data[1])*accelScale))))
		s.AccelZ = int16(math.Round(min(float64(math.MaxInt16), max(float64(math.MinInt16), float64(-data[2])*accelScale))))
	}
}
